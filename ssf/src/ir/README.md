# ssf intermediate representation (IR)

## Design decisions

### Heap allocation

In the ssf IR, heap allocation happens in the following cases:

- To create boxed constructors of ADTs.
- To create closures in let-recursive expressions.
