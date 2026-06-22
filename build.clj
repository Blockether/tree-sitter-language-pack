(ns build
  "Build & deploy the JVM binding of tree-sitter-language-pack to Clojars.

  Same packaging model as Blockether/rift-clojure: the main
  `com.blockether/tree-sitter-language-pack` jar is small (Java binding classes +
  the runtime native loader) and bundles NO native libraries. Each platform's
  native FFI library is published as its own jar,
  `com.blockether/tree-sitter-language-pack-native-<rid>`, carrying the lib as a
  classpath resource under `natives/<rid>/` — exactly where NativeLib looks it up
  at runtime. Consumers add the main jar plus the single native jar for their
  platform, instead of downloading every platform's library.

  Consumers must run the JVM with `--enable-native-access=ALL-UNNAMED`.

  Usage:
    VERSION=v1.2.3 clojure -T:build deploy                       ; main jar -> Clojars
    VERSION=v1.2.3 clojure -T:build deploy-native :rid macos-arm64
    clojure -T:build install                                      ; main jar -> ~/.m2"
  (:require [clojure.java.io :as io]
            [clojure.string :as str]
            [clojure.tools.build.api :as b]
            [deps-deploy.deps-deploy :as dd]))

(def lib 'com.blockether/tree-sitter-language-pack)

;; rid values match NativeLib.resolveNativesRid (os-arch).
(def native-rids #{"linux-x86_64" "linux-aarch64" "macos-arm64" "macos-x86_64" "windows-x86_64"})
(def native-libs {"linux-x86_64"   "libts_pack_core_ffi.so"
                  "linux-aarch64"  "libts_pack_core_ffi.so"
                  "macos-arm64"    "libts_pack_core_ffi.dylib"
                  "macos-x86_64"   "libts_pack_core_ffi.dylib"
                  "windows-x86_64" "ts_pack_core_ffi.dll"})

(defn- cargo-version []
  (some->> (slurp "Cargo.toml")
           (re-find #"(?m)^\s*version\s*=\s*\"([^\"]+)\"")
           second))

(def version
  (let [v (System/getenv "VERSION")]
    (cond
      (and v (str/starts-with? v "v")) (subs v 1)
      (and v (seq v))                  v
      :else                            (or (cargo-version) "0.0.1-SNAPSHOT"))))

(def class-dir "target/classes")
(def jar-file (format "target/%s-%s.jar" (name lib) version))
(def java-src "packages/java/src/main/java")
(def java-resources "packages/java/src/main/resources")
;; Clojure native-resolver source (com.blockether.tree-sitter-language-pack),
;; shipped in the main jar so requiring it auto-selects the platform native.
(def clj-src "src/clj")
;; Per-platform native libs are staged here as <rid>/<lib> (by CI from the
;; native build artifacts, or locally via cargo + manual copy).
(def native-staging "target/native-staging")

;; Drop org.clojure/clojure from the basis: it is only the build-tool runtime,
;; not a dependency of this Java library, and must not leak into the published pom.
(def basis (delay (-> (b/create-basis {:project "deps.edn"})
                      (update :libs dissoc 'org.clojure/clojure))))

(defn clean [_]
  (b/delete {:path "target"}))

(defn- pom-data [description]
  [[:description description]
   [:url "https://github.com/Blockether/tree-sitter-language-pack"]
   [:licenses
    [:license
     [:name "MIT"]
     [:url "https://opensource.org/licenses/MIT"]]]
   [:scm
    [:url "https://github.com/Blockether/tree-sitter-language-pack"]
    [:connection "scm:git:git://github.com/Blockether/tree-sitter-language-pack.git"]
    [:developerConnection "scm:git:ssh://git@github.com/Blockether/tree-sitter-language-pack.git"]]])

(defn compile-java [_]
  (b/javac {:src-dirs   [java-src]
            :class-dir  class-dir
            :basis      @basis
            ;; alef emits JDK 25 FFM bindings that require preview features.
            :javac-opts ["--release" "25" "--enable-preview"]}))

(defn jar
  "Build the main jar: Java binding classes + resources, NO native libraries."
  [_]
  (clean nil)
  (compile-java nil)
  (b/write-pom {:class-dir class-dir
                :lib lib
                :version version
                :basis @basis
                :src-dirs [java-src]
                :pom-data (pom-data "Pre-compiled tree-sitter grammars — JVM binding, consumable from Clojure. Add a com.blockether/tree-sitter-language-pack-native-<rid> jar for your platform's native library.")})
  (b/copy-dir {:src-dirs [java-resources clj-src] :target-dir class-dir})
  ;; Version resource read by the Clojure resolver to locate the matching
  ;; per-rid native jar on Clojars.
  (spit (io/file class-dir "tslp-version") version)
  ;; Guard: the main jar must never carry a native library.
  (doseq [ext ["so" "dylib" "dll"]
          f (->> (io/file class-dir) file-seq (filter #(.isFile ^java.io.File %)))
          :when (str/ends-with? (.getName ^java.io.File f) (str "." ext))]
    (throw (ex-info (str "Native library leaked into the main jar: " f
                         " — natives belong only in the per-rid native jars.") {:file (str f)})))
  (b/jar {:class-dir class-dir :jar-file jar-file})
  (println "Built main jar:" jar-file "version:" version))

(defn- native-lib-sym [rid]
  (symbol "com.blockether" (str "tree-sitter-language-pack-native-" rid)))

(defn native-jar
  "Build the native jar for one rid: contains only natives/<rid>/<lib>."
  [{:keys [rid]}]
  (let [rid   (some-> rid name)
        _     (when-not (native-rids rid)
                (throw (ex-info (str "Unknown rid: " rid) {:rid rid :known native-rids})))
        fname (native-libs rid)
        src   (format "%s/%s/%s" native-staging rid fname)
        lib*  (native-lib-sym rid)
        jar*  (format "target/%s-%s.jar" (name lib*) version)
        cdir  (format "target/native-classes-%s" rid)]
    (b/delete {:path cdir})
    (b/delete {:path jar*})
    (when-not (.exists (io/file src))
      (throw (ex-info (str "Native library not found: " src
                           ". Build it (cargo build -p ts-pack-core-ffi --release) and stage it at "
                           native-staging "/" rid "/" fname) {:rid rid :path src})))
    (b/write-pom {:class-dir cdir
                  :lib lib*
                  :version version
                  :basis @basis
                  :src-dirs []
                  :pom-data (pom-data (format "Native tree-sitter FFI library (libts_pack_core_ffi) for %s." rid))})
    (b/copy-file {:src src :target (format "%s/natives/%s/%s" cdir rid fname)})
    (b/jar {:class-dir cdir :jar-file jar*})
    (println "Built native jar:" jar* "version:" version)
    {:jar jar* :lib lib* :class-dir cdir}))

(defn deploy
  "Build and deploy the main jar to Clojars."
  [_]
  (jar nil)
  (dd/deploy {:installer :remote
              :artifact jar-file
              :pom-file (b/pom-path {:lib lib :class-dir class-dir})}))

(defn deploy-native
  "Build and deploy one platform's native jar to Clojars. Pass :rid <rid>."
  [{:keys [rid]}]
  (let [{:keys [jar lib class-dir]} (native-jar {:rid rid})]
    (dd/deploy {:installer :remote
                :artifact jar
                :pom-file (b/pom-path {:lib lib :class-dir class-dir})})))

(defn install
  "Build and install the main jar into the local Maven repo (~/.m2)."
  [_]
  (jar nil)
  (dd/deploy {:installer :local
              :artifact jar-file
              :pom-file (b/pom-path {:lib lib :class-dir class-dir})}))
