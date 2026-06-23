(ns com.blockether.tree-sitter-language-pack
  "Auto-resolves the platform-specific native FFI library for
   com.blockether/tree-sitter-language-pack — the same pattern as
   Blockether/rift-clojure.

   Requiring this namespace selects the right native for the current OS/arch
   so consumers add a SINGLE dependency
   (`com.blockether/tree-sitter-language-pack`) instead of also picking a
   per-platform native jar by hand.

   Resolution order (first hit wins), mirroring rift:

     1. an explicit path — `TSLP_NATIVE_PATH` env or
        `dev.kreuzberg.treesitterlanguagepack.native.path` system property,
     2. a bundled `natives/<rid>/<lib>` classpath resource (e.g. when the
        matching `…-native-<rid>` jar is already on the classpath),
     3. otherwise resolve `com.blockether/tree-sitter-language-pack-native-<rid>`
        through Clojure's own dependency machinery (`clojure.tools.deps`, the
        same resolver the `clojure` CLI uses — so your `:mvn/repos`, mirrors,
        `~/.m2/settings.xml`, and local repository are all honoured), extract the
        library out of the resolved jar into `~/.cache/clj-tslp`, and point
        NativeLib at it via the system property.

   Require this namespace BEFORE using the Java API
   (dev.kreuzberg.treesitterlanguagepack.TreeSitterLanguagePack), and run the
   JVM with `--enable-native-access=ALL-UNNAMED`."
  (:require [clojure.java.io :as io]
            [clojure.string :as str])
  (:import [java.nio.file CopyOption Files LinkOption Path StandardCopyOption]
           [java.nio.file.attribute FileAttribute]
           [java.util.jar JarFile]))

(set! *warn-on-reflection* true)

(def ^:private native-prop "com.blockether.treesitterlanguagepack.native.path")
(def ^:private no-link-options (make-array LinkOption 0))
(def ^:private no-file-attributes (make-array FileAttribute 0))

(defn rid
  "Runtime identifier for this platform, matching NativeLib.resolveNativesRid
   (e.g. \"macos-arm64\", \"linux-x86_64\", \"windows-x86_64\")."
  []
  (let [os   (str/lower-case (System/getProperty "os.name" ""))
        arch (str/lower-case (System/getProperty "os.arch" ""))
        os*  (cond
               (or (str/includes? os "mac") (str/includes? os "darwin")) "macos"
               (str/includes? os "win")                                  "windows"
               :else                                                     "linux")
        arch* (cond
                (or (str/includes? arch "aarch64") (str/includes? arch "arm64"))
                (if (= os* "macos") "arm64" "aarch64")
                (or (str/includes? arch "x86_64") (str/includes? arch "amd64")) "x86_64"
                :else (throw (ex-info (str "Unsupported architecture: " arch) {:arch arch})))]
    (str os* "-" arch*)))

(defn- lib-filename [rid]
  (cond
    (str/starts-with? rid "windows") "ts_pack_core_ffi.dll"
    (str/starts-with? rid "macos")   "libts_pack_core_ffi.dylib"
    :else                            "libts_pack_core_ffi.so"))

(defn- version []
  (some-> (io/resource "tslp-version") slurp str/trim not-empty))

(defn- cache-dir ^Path [version rid]
  (.toPath (io/file (or (System/getenv "TSLP_CACHE_DIR")
                        (str (io/file (System/getProperty "user.home") ".cache" "clj-tslp")))
                    version rid)))

(defn- resolve-native-jar ^Path [version rid]
  "Resolve the per-rid native jar through `clojure.tools.deps` — the same
   resolver the `clojure` CLI uses, so configured Maven repositories, mirrors and
   `~/.m2/settings.xml` are all honoured (no hand-rolled HTTP to a hardcoded
   repo). Returns the path of the jar in the local Maven repository.

   tools.deps is loaded via `requiring-resolve` so it is only touched on this
   runtime download path (never under native-image, where the native is bundled
   or supplied via an explicit path)."
  (let [lib          (symbol "com.blockether" (str "tree-sitter-language-pack-native-" rid))
        create-basis (or (requiring-resolve 'clojure.tools.deps/create-basis)
                         (throw (ex-info "org.clojure/tools.deps is not on the classpath; cannot resolve the native artifact. Add the matching native jar to the classpath or set TSLP_NATIVE_PATH."
                                  {:lib lib})))
        basis        (create-basis {:project nil :extra {:deps {lib {:mvn/version version}}}})
        path         (-> basis :libs (get lib) :paths first)]
    (when-not path
      (throw (ex-info (str "Could not resolve native artifact " lib " " version
                        " via Clojure's dependency resolver. Check your Maven repositories / mirrors.")
               {:lib lib :version version})))
    (.toPath (io/file path))))

(defn- extract! [^Path jar-path ^String entry-name ^Path dest]
  (Files/createDirectories (.getParent dest) no-file-attributes)
  (with-open [jar (JarFile. (.toFile jar-path))]
    (let [entry (.getEntry jar entry-name)]
      (when-not entry
        (throw (ex-info (str "Native jar " jar-path " is missing entry " entry-name)
                        {:jar (str jar-path) :entry entry-name})))
      (with-open [in ^java.io.InputStream (.getInputStream jar entry)]
        (Files/copy in dest
                    ^"[Ljava.nio.file.CopyOption;"
                    (into-array CopyOption [StandardCopyOption/REPLACE_EXISTING])))))
  dest)

(defn ensure-native!
  "Idempotently make the platform native FFI library available and point
   NativeLib at it. Returns the resolved library path as a string, or nil when
   loading is left to NativeLib (explicit override already set, or the native
   is bundled on the classpath). Safe to call repeatedly."
  []
  (cond
    ;; (1) explicit override already in effect — NativeLib will honour it.
    (or (System/getProperty native-prop) (System/getenv "TSLP_NATIVE_PATH"))
    (System/getProperty native-prop)

    :else
    (let [rid   (rid)
          fname (lib-filename rid)]
      (if (io/resource (str "natives/" rid "/" fname))
        ;; (2) bundled on the classpath — let NativeLib extract+load it.
        nil
        ;; (3) resolve the matching native jar via Clojure's dependency resolver
        ;; (tools.deps — honours repos/mirrors/settings.xml) and cache the lib.
        (let [v   (or (version)
                      (throw (ex-info "Cannot determine artifact version (missing tslp-version resource)" {})))
              ^Path dir (cache-dir v rid)
              ^Path lib (.resolve dir ^String fname)]
          (when-not (Files/exists lib no-link-options)
            (extract! (resolve-native-jar v rid) (str "natives/" rid "/" fname) lib))
          (let [abs (str (.toAbsolutePath lib))]
            (System/setProperty native-prop abs)
            abs))))))

;; Resolve on load so simply requiring this namespace is enough on a normal JVM.
;; Skipped during AOT compilation (Clojure binds *compile-files* true) so a
;; GraalVM native-image build does NOT perform a build-time download; under
;; native-image, call `ensure-native!` yourself at runtime (e.g. at the top of
;; -main) before touching the Java API.
(when-not *compile-files*
  (ensure-native!))
