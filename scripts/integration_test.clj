;; JVM integration gate for the deploy pipeline — the test vis's suite used to
;; be the only line of defence for. Exercises the FRESHLY BUILT native library
;; (TSLP_NATIVE_PATH) through the same alef/FFM binding consumers use:
;;
;;   1. STATIC grammar  — clojure (the fork's own grammar override) parses and
;;      yields structure.
;;   2. INTEL           — elixir structure extraction (module + public/private
;;      fns), the per-language intelligence the fork exists for.
;;   3. DYNAMIC grammar — haskell is NOT in TSLP_LANGUAGES, so getLanguage
;;      must fetch the manifest + bundle from the pinned release
;;      (DYNAMIC_ASSETS_VERSION). CI runners are cold, so this validates the
;;      whole download path against the live network — the exact path that
;;      silently broke in 1.12.3-blockether.1/.2/.3.
;;
;; Run: TSLP_NATIVE_PATH=<lib> clojure -M:it   (see deploy-clojars.yml)
(ns integration-test
  (:import [dev.kreuzberg.treesitterlanguagepack
            TreeSitterLanguagePack ProcessConfig StructureItem Parser Tree]))

(defn- fail! [msg]
  (binding [*out* *err*] (println "INTEGRATION FAIL:" msg))
  (System/exit 1))

(defn- cause-chain [^Throwable t]
  (clojure.string/join " <- " (map str (take-while some? (iterate #(.getCause ^Throwable %) t)))))

(when-not (System/getenv "TSLP_NATIVE_PATH")
  (fail! "TSLP_NATIVE_PATH not set — point it at the freshly built libts_pack_core_ffi"))

(defn- structure [source lang]
  (let [cfg (-> (ProcessConfig/builder) (.withLanguage lang) (.withStructure true) (.build))]
    (vec (or (.structure (TreeSitterLanguagePack/process source cfg)) []))))

;; 1. STATIC: clojure
(let [items (structure "(defn greet [x] x)\n(def answer 42)\n" "clojure")
      names (set (map #(.name ^StructureItem %) items))]
  (when-not (names "greet")
    (fail! (str "static clojure structure missing 'greet' — got " names)))
  (println "OK static/clojure:" names))

;; 2. INTEL: elixir (module + visibility)
(let [src   "defmodule Greeter do\n  def hello(n), do: n\n  defp secret, do: 42\nend\n"
      items (structure src "elixir")
      mod   (first items)
      kids  (when mod (vec (or (.children ^StructureItem mod) [])))
      by-nm (into {} (map (fn [^StructureItem i] [(.name i) (.visibility i)])) kids)]
  (when-not (and mod (= "Greeter" (.name ^StructureItem mod)))
    (fail! (str "elixir intel: expected module Greeter — got " (mapv str items))))
  (when-not (= "public" (get by-nm "hello"))
    (fail! (str "elixir intel: hello should be public — got " by-nm)))
  (when-not (= "private" (get by-nm "secret"))
    (fail! (str "elixir intel: secret should be private — got " by-nm)))
  (println "OK intel/elixir:" by-nm))

;; 3. DYNAMIC: haskell is NOT in TSLP_LANGUAGES — Parser.setLanguage goes
;; through the Rust-side dynamic loader: manifest + grammar bundle fetched
;; from the pinned release (DYNAMIC_ASSETS_VERSION). CI runners are cold, so
;; this validates the whole download path against the live network — the
;; exact path that silently broke in 1.12.3-blockether.1/.2/.3. NOTE: the
;; pack's own Parser (Rust-side) is the consumer path (vis zipper); the
;; jtreesitter getLanguage wrapper additionally needs a standalone
;; libtree-sitter and is NOT what this gate covers.
(let [p (Parser/create)]
  (try
    (.setLanguage p "haskell")
    (let [^Tree tree (.orElse (.parse p "add a b = a + b\n") nil)]
      (when (nil? tree) (fail! "dynamic haskell parse returned nil"))
      (.close tree)
      (println "OK dynamic/haskell: grammar downloaded + parsed"))
    (catch Throwable t (fail! (str "dynamic haskell: " (cause-chain t))))
    (finally (.close p))))

(println "INTEGRATION OK")
(System/exit 0)
