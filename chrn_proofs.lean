/- Prove that there is no e in chrn -/
/- calc is short for calculator for anyone new -/

def c : Nat := 99
def h : Nat := 104
def e : Nat := 101
def r : Nat := 114
def n : Nat := 110

def chrn : Nat := c + h + r + n
def chern : Nat := c + h + e + r + n

/- Proves 'e' ∉ "chrn" -/
theorem chrn_has_no_e : chrn ≠ chern := by
    decide

/-
TODO: Prove `TypeResolver`'s dependency resolution system

-/
