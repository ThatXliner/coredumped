//! Prelude: standard library functions implemented in Glyph itself.
//!
//! This module contains the Glyph source for library functions that would
//! otherwise need to be Rust builtins. They're loaded into the environment
//! at startup so you can see how they work (and edit them in-game).

pub const SOURCE: &str = r#";; ---- Glyph Prelude ---------------------------------------------------
;; Sequence operations built from cons/first/rest/empty?

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

;; Multi-arity range — demonstrates the (fn ([x] ...) ([x y] ...)) syntax
(const range (fn ([end]       (range-internal 0 end 1))
                  ([s e]     (range-internal s e 1))
                  ([s e st]  (range-internal s e st))))

(const range-internal (fn [start end step]
  (if (if (< step 0) (> start end) (< start end))
      (cons start (range-internal (+ start step) end step))
      (list))))

;; (repeat n expr...) evaluates the body expressions n times in sequence
(defmacro repeat [n & body]
  (list 'map
        (apply list 'fn (cons (list '_) body))
        (list 'range n)))

;; (lambda body)         → zero-arg function with one body expression
;; (lambda [params] body...) → alias for (fn [params] body...)
(defmacro lambda [params & body]
  (if (empty? body)
      (list 'fn [] (list params))
      (cons 'fn (cons params body))))
"#;
