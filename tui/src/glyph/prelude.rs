//! Prelude: standard library functions implemented in Glyph itself.
//!
//! This module contains the Glyph source for library functions that would
//! otherwise need to be Rust builtins. They're loaded into the environment
//! at startup so you can see how they work (and edit them in-game).

pub const SOURCE: &str = r#";; ---- Glyph Prelude ---------------------------------------------------
;; Sequence operations built from cons/first/rest/empty?
;;
;; Note: Glyph's `let` is single-binding: (let name value body...)
;; Use nested lets instead of vector bindings.

(const second (fn [lst]
  (first (rest lst))))

(const nth (fn [lst n]
  (if (= n 0)
      (first lst)
      (nth (rest lst) (- n 1)))))

(const filter (fn [pred lst]
  (if (empty? lst)
      (list)
      (let x (first lst)
        (if (pred x)
            (cons x (filter pred (rest lst)))
            (filter pred (rest lst)))))))

(const reduce (fn [f init lst]
  (if (empty? lst)
      init
      (let new-init (f init (first lst))
        (reduce f new-init (rest lst))))))

(const some (fn [pred lst]
  (if (empty? lst)
      nil
      (let val (pred (first lst))
        (if val val (some pred (rest lst)))))))

(const every (fn [pred lst]
  (if (empty? lst)
      true
      (if (pred (first lst))
          (every pred (rest lst))
          false))))

(const take (fn [n lst]
  (if (or (= n 0) (empty? lst))
      (list)
      (cons (first lst) (take (- n 1) (rest lst))))))

(const drop (fn [n lst]
  (if (= n 0)
      lst
      (drop (- n 1) (rest lst)))))

(const append (fn [lst x]
  (concat lst (list x))))

(const concat (fn [a b]
  (if (empty? a)
      b
      (cons (first a) (concat (rest a) b)))))

(const reverse (fn [lst]
  (reverse-acc lst (list))))

(const reverse-acc (fn [lst acc]
  (if (empty? lst)
      acc
      (reverse-acc (rest lst) (cons (first lst) acc)))))

;; (range-internal start end step)
;; Bound as `range` at load time to shadow the Rust builtin.
(const range-internal (fn [start end step]
  (if (if (< step 0) (> start end) (< start end))
      (cons start (range-internal (+ start step) end step))
      (list))))

;; Multi-arity entry point: (range end) | (range start end) | (range start end step)
(const range-entry (fn [& args]
  (let a (first args)
    (let rest-1 (rest args)
      (if (empty? rest-1)
          (range-internal 0 a 1)
          (let b (first rest-1)
            (let rest-2 (rest rest-1)
              (if (empty? rest-2)
                  (range-internal a b 1)
                  (let c (first rest-2)
                    (range-internal a b c))))))))))
"#;
