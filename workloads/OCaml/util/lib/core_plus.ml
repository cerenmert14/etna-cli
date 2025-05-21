open Runner

(* Core provides functions for composition of generators. These are some functions for composition of module types. *)
(* todo: i have no clue what the observers do *)

let double (type a b) (module M1 : Base_quickcheck.Test.S with type t = a)
    (module M2 : Base_quickcheck.Test.S with type t = b) : (a * b) basegen =
  (module struct
    let sexp_of_a = M1.sexp_of_t
    let sexp_of_b = M2.sexp_of_t
    let quickcheck_generator_a = M1.quickcheck_generator
    let quickcheck_generator_b = M2.quickcheck_generator
    let quickcheck_shrinker_a = M1.quickcheck_shrinker
    let quickcheck_shrinker_b = M2.quickcheck_shrinker
    let quickcheck_observer_a = Base_quickcheck.Observer.opaque
    let quickcheck_observer_b = Base_quickcheck.Observer.opaque

    type t = a * b [@@deriving sexp_of, quickcheck]
  end)

let triple (type a b c) (module M1 : Base_quickcheck.Test.S with type t = a)
    (module M2 : Base_quickcheck.Test.S with type t = b)
    (module M3 : Base_quickcheck.Test.S with type t = c) : (a * b * c) basegen =
  (module struct
    let sexp_of_a = M1.sexp_of_t
    let sexp_of_b = M2.sexp_of_t
    let sexp_of_c = M3.sexp_of_t
    let quickcheck_generator_a = M1.quickcheck_generator
    let quickcheck_generator_b = M2.quickcheck_generator
    let quickcheck_generator_c = M3.quickcheck_generator
    let quickcheck_shrinker_a = M1.quickcheck_shrinker
    let quickcheck_shrinker_b = M2.quickcheck_shrinker
    let quickcheck_shrinker_c = M3.quickcheck_shrinker
    let quickcheck_observer_a = Base_quickcheck.Observer.opaque
    let quickcheck_observer_b = Base_quickcheck.Observer.opaque
    let quickcheck_observer_c = Base_quickcheck.Observer.opaque

    type t = a * b * c [@@deriving sexp_of, quickcheck]
  end)

let quad (type a b c d) (module M1 : Base_quickcheck.Test.S with type t = a)
    (module M2 : Base_quickcheck.Test.S with type t = b)
    (module M3 : Base_quickcheck.Test.S with type t = c)
    (module M4 : Base_quickcheck.Test.S with type t = d) :
    (a * b * c * d) basegen =
  (module struct
    let sexp_of_a = M1.sexp_of_t
    let sexp_of_b = M2.sexp_of_t
    let sexp_of_c = M3.sexp_of_t
    let sexp_of_d = M4.sexp_of_t
    let quickcheck_generator_a = M1.quickcheck_generator
    let quickcheck_generator_b = M2.quickcheck_generator
    let quickcheck_generator_c = M3.quickcheck_generator
    let quickcheck_generator_d = M4.quickcheck_generator
    let quickcheck_shrinker_a = M1.quickcheck_shrinker
    let quickcheck_shrinker_b = M2.quickcheck_shrinker
    let quickcheck_shrinker_c = M3.quickcheck_shrinker
    let quickcheck_shrinker_d = M4.quickcheck_shrinker
    let quickcheck_observer_a = Base_quickcheck.Observer.opaque
    let quickcheck_observer_b = Base_quickcheck.Observer.opaque
    let quickcheck_observer_c = Base_quickcheck.Observer.opaque
    let quickcheck_observer_d = Base_quickcheck.Observer.opaque

    type t = a * b * c * d [@@deriving sexp_of, quickcheck]
  end)

let quinta (type a b c d e) (module M1 : Base_quickcheck.Test.S with type t = a)
    (module M2 : Base_quickcheck.Test.S with type t = b)
    (module M3 : Base_quickcheck.Test.S with type t = c)
    (module M4 : Base_quickcheck.Test.S with type t = d)
    (module M5 : Base_quickcheck.Test.S with type t = e) :
    (a * b * c * d * e) basegen =
  (module struct
    let sexp_of_a = M1.sexp_of_t
    let sexp_of_b = M2.sexp_of_t
    let sexp_of_c = M3.sexp_of_t
    let sexp_of_d = M4.sexp_of_t
    let sexp_of_e = M5.sexp_of_t
    let quickcheck_generator_a = M1.quickcheck_generator
    let quickcheck_generator_b = M2.quickcheck_generator
    let quickcheck_generator_c = M3.quickcheck_generator
    let quickcheck_generator_d = M4.quickcheck_generator
    let quickcheck_generator_e = M5.quickcheck_generator
    let quickcheck_shrinker_a = M1.quickcheck_shrinker
    let quickcheck_shrinker_b = M2.quickcheck_shrinker
    let quickcheck_shrinker_c = M3.quickcheck_shrinker
    let quickcheck_shrinker_d = M4.quickcheck_shrinker
    let quickcheck_shrinker_e = M5.quickcheck_shrinker
    let quickcheck_observer_a = Base_quickcheck.Observer.opaque
    let quickcheck_observer_b = Base_quickcheck.Observer.opaque
    let quickcheck_observer_c = Base_quickcheck.Observer.opaque
    let quickcheck_observer_d = Base_quickcheck.Observer.opaque
    let quickcheck_observer_e = Base_quickcheck.Observer.opaque

    type t = a * b * c * d * e [@@deriving sexp_of, quickcheck]
  end)
