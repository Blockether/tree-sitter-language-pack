(ns build
  "Build & deploy the JVM binding of tree-sitter-language-pack to Clojars.

  Same approach as Blockether/svar: compile the alef-generated Java sources,
  bundle the prebuilt native FFI library/-ies, write a pom, jar, and deploy via
  deps-deploy. The native lib(s) must already exist under
  packages/java/src/main/resources/natives/<rid>/ before `jar`/`deploy`/`install`
  (CI builds them with `cargo build -p ts-pack-core-ffi --release`; see
  .github/workflows/deploy-clojars.yml). Locally you can stage one platform via
  the `stage-natives` task.

  Usage:
    VERSION=v1.10.3 clojure -T:build deploy   ; build + deploy to Clojars
    clojure -T:build install                  ; build + install into ~/.m2
    clojure -T:build jar                       ; just build the jar"
  (:require [clojure.tools.build.api :as b]
            [clojure.java.io :as io]
            [clojure.string :as str]
            [deps-deploy.deps-deploy :as dd]))

(def lib 'com.blockether/tree-sitter-language-pack)

(defn- cargo-version []
  ;; Canonical version lives in the Cargo workspace.
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
;; Drop org.clojure/clojure from the basis: it is only the build-tool runtime,
;; not a dependency of this Java library, and must not leak into the published pom.
(def basis (delay (-> (b/create-basis {:project "deps.edn"})
                      (update :libs dissoc 'org.clojure/clojure))))

(defn clean [_]
  (b/delete {:path "target"}))

(defn compile-java [_]
  (b/javac {:src-dirs   [java-src]
            :class-dir  class-dir
            :basis      @basis
            ;; alef emits JDK 25 FFM bindings that require preview features.
            :javac-opts ["--release" "25" "--enable-preview"]}))

(defn jar [_]
  (clean nil)
  (compile-java nil)
  (b/write-pom {:class-dir class-dir
                :lib lib
                :version version
                :basis @basis
                :src-dirs [java-src]
                :pom-data [[:description "Pre-compiled tree-sitter grammars for 306 programming languages — JVM binding, consumable from Clojure."]
                           [:url "https://github.com/Blockether/tree-sitter-language-pack"]
                           [:licenses
                            [:license
                             [:name "MIT"]
                             [:url "https://opensource.org/licenses/MIT"]]]
                           [:scm
                            [:url "https://github.com/Blockether/tree-sitter-language-pack"]
                            [:connection "scm:git:git://github.com/Blockether/tree-sitter-language-pack.git"]
                            [:developerConnection "scm:git:ssh://git@github.com/Blockether/tree-sitter-language-pack.git"]]]})
  ;; Bundle compiled classes + resources (which include natives/<rid>/<lib>).
  (b/copy-dir {:src-dirs [java-resources]
               :target-dir class-dir})
  (b/jar {:class-dir class-dir
          :jar-file jar-file})
  (let [natives (->> (file-seq (java.io.File. (str java-resources "/natives")))
                     (filter #(.isFile ^java.io.File %))
                     (map #(.getPath ^java.io.File %))
                     (filter #(re-find #"libts_pack_core_ffi|ts_pack_core_ffi\.dll" %)))]
    (println "Bundled native libs:" (if (seq natives) (vec natives) "NONE (jar will rely on java.library.path / runtime download)")))
  (println "Built:" jar-file "version:" version))

(defn- current-rid
  "Runtime identifier matching NativeLib.resolveNativesRid (os-arch)."
  []
  (let [os   (str/lower-case (System/getProperty "os.name" ""))
        arch (str/lower-case (System/getProperty "os.arch" ""))
        os*  (cond (or (str/includes? os "mac") (str/includes? os "darwin")) "macos"
                   (str/includes? os "win")                                  "windows"
                   :else                                                     "linux")
        arch* (cond (or (str/includes? arch "aarch64") (str/includes? arch "arm64"))
                    (if (= os* "macos") "arm64" "aarch64")
                    (or (str/includes? arch "x86_64") (str/includes? arch "amd64")) "x86_64"
                    :else (str/replace arch #"[^a-z0-9_]+" ""))]
    (str os* "-" arch*)))

(defn- native-filename []
  (let [os (str/lower-case (System/getProperty "os.name" ""))]
    (cond (str/includes? os "win")                                  "ts_pack_core_ffi.dll"
          (or (str/includes? os "mac") (str/includes? os "darwin")) "libts_pack_core_ffi.dylib"
          :else                                                     "libts_pack_core_ffi.so")))

(defn stage-natives
  "Copy the locally built native FFI lib (target/release/<lib>) into
   packages/java/src/main/resources/natives/<rid>/ for the host platform, so a
   subsequent `jar`/`install` bundles it. Build the lib first, e.g.:
     TSLP_LANGUAGES=clojure TSLP_LINK_MODE=static \\
       cargo build -p ts-pack-core-ffi --release"
  [_]
  (let [fname    (native-filename)
        src      (io/file "target/release" fname)
        dest-dir (io/file java-resources "natives" (current-rid))
        dest     (io/file dest-dir fname)]
    (when-not (.exists src)
      (throw (ex-info (str "Native lib not found: " src
                           ". Build it first: cargo build -p ts-pack-core-ffi --release")
                      {:expected (str src)})))
    (.mkdirs dest-dir)
    (b/copy-file {:src (str src) :target (str dest)})
    (println "Staged" (str src) "->" (str dest))))

(defn deploy [_]
  (jar nil)
  (dd/deploy {:installer :remote
              :artifact jar-file
              :pom-file (b/pom-path {:lib lib :class-dir class-dir})}))

(defn install
  "Build and install the jar into the local Maven repo (~/.m2)."
  [_]
  (jar nil)
  (dd/deploy {:installer :local
              :artifact jar-file
              :pom-file (b/pom-path {:lib lib :class-dir class-dir})}))
