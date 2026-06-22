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
     3. otherwise download `com.blockether/tree-sitter-language-pack-native-<rid>`
        from Clojars into `~/.cache/clj-tslp`, extract the library, and point
        NativeLib at it via the system property.

   Require this namespace BEFORE using the Java API
   (dev.kreuzberg.treesitterlanguagepack.TreeSitterLanguagePack), and run the
   JVM with `--enable-native-access=ALL-UNNAMED`."
  (:require [clojure.java.io :as io]
            [clojure.string :as str])
  (:import [java.net URI]
           [java.net.http HttpClient HttpRequest HttpResponse$BodyHandlers]
           [java.nio.file CopyOption Files LinkOption Path Paths StandardCopyOption]
           [java.nio.file.attribute FileAttribute]
           [java.util.jar JarFile]))

(set! *warn-on-reflection* true)

(def ^:private native-prop "dev.kreuzberg.treesitterlanguagepack.native.path")
(def ^:private clojars-root "https://repo.clojars.org")
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

(defn- native-jar-uri ^URI [version rid]
  (let [artifact (str "tree-sitter-language-pack-native-" rid)]
    (URI/create (format "%s/com/blockether/%s/%s/%s-%s.jar"
                        clojars-root artifact version artifact version))))

(defn- download! [^URI uri ^Path dest]
  (Files/createDirectories (.getParent dest) no-file-attributes)
  (Files/deleteIfExists dest)
  (let [client   (HttpClient/newHttpClient)
        request  (-> (HttpRequest/newBuilder uri) (.GET) (.build))
        response (.send client request (HttpResponse$BodyHandlers/ofFile dest))]
    (when-not (= 200 (.statusCode response))
      (Files/deleteIfExists dest)
      (throw (ex-info (str "Unable to download native artifact from " uri
                           " (HTTP " (.statusCode response) ")")
                      {:uri (str uri) :status (.statusCode response)})))
    dest))

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
        ;; (3) download the matching native jar from Clojars and cache it.
        (let [v   (or (version)
                      (throw (ex-info "Cannot determine artifact version (missing tslp-version resource)" {})))
              ^Path dir (cache-dir v rid)
              ^Path lib (.resolve dir ^String fname)]
          (when-not (Files/exists lib no-link-options)
            (let [^Path jar (.resolve dir (str "tree-sitter-language-pack-native-" rid ".jar"))]
              (download! (native-jar-uri v rid) jar)
              (extract! jar (str "natives/" rid "/" fname) lib)))
          (let [abs (str (.toAbsolutePath lib))]
            (System/setProperty native-prop abs)
            abs))))))

;; Resolve on load so simply requiring this namespace is enough.
(ensure-native!)
