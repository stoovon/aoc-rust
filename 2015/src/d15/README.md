# Day 15: Science for Hungry People

Both parts validate unique ingredient names and exactly one of each named property:
capacity, durability, flavor, texture and calories. Properties may appear in any
order and use signed i64 values. Names are nonempty ASCII letters, digits or
underscores. Malformed, unknown, missing or duplicate fields return errors with
line context. Empty input and duplicate ingredient names are rejected.

The shared search assigns a nonnegative teaspoon count to each ingredient,
forcing the final ingredient to take the remainder. It carries small Copy
attribute totals and borrows the ingredient slice; no 100-element ingredient
lists, cloned names or per-recipe hash maps are built.

Part 1 imposes no calorie requirement. Part 2 requires exactly 500 calories.
A feasible recipe with a zero quality score returns zero; no calorie-compatible
recipe returns an error. Nonpositive quality totals clamp the score to zero;
calories are summed without clamping. Scaling, totals and products use checked
i64 arithmetic. The calorie check precedes the quality product.

For T teaspoons and m ingredients, the search visits C(T+m-1, m-1) complete
quantity assignments using O(m) search storage. It remains combinatorial in the
number of ingredients. A budget of 10,000,000 assignments is checked before search;
at T=100 this supports up to five ingredients, with an error beyond that budget.
