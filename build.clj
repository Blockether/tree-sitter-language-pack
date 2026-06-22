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
