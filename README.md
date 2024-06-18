# Execution cost estimator for LBF circuits
Estimate execution cost of a LBF files using concrete estimation tools

## Initialize pathed concrete library
```bash
bash init.sh
```

## Estimation examples

Estimate sequential execution time:
```bash
python3 estimator.py sample.lbf
```

Change functional bootstrapping precision:
```bash
python3 estimator.py sample.lbf --fbs_size 6
```

## LBF format

The LBF (Lincomb Bootstrap Format) files encode circuits composed of linear combinations and bootstrapping gates.
LBF files are used to express Boolean circuits in the TFHE functional bootstrapping model of computation.
The LBF format is inspired from BLIF format for logic circuits.

LBF files has the following structure:
```
.inputs <input list>
.outputs <output list>
<gate list>
.end
```

Circuit inputs and outputs are defined as a list of string identifiers and contain at least one element each.
Two types of nodes are defined: linear combination and bootstrapping nodes.

### Linear combination node

A linear combination node is defined as:
```
.lincomb <input list> <output>
<coefficient list> [<constant coefficient>]
```

The size of input and coefficient list are equal.
Optionally a constant coefficient can be declared, in case it is not given zero value is used.
If the input list is empty then the output is a constant, given by the constant coefficient.

#### Examples

A linear combination `c = 2 * a + b`

```
.lincomb a b c
2 1

```


A linear combination `c = a - b + 1`

```
.lincomb a b c
1 -1 1

```

A constant `c = 2`
```
.lincomb c
2
```


### Bootstrapping node

A bootstrapping node is defined as:
```
.bootstrap <input> <output list>
<tables>
```

This node maps the input value _k_ to the _k_-th entry of the corresponding table.
One table per output in the output list is given.
A table is a string of `0` and `1` whose length is equal to the maximum span on input.


#### Examples


A bootstrapping `b = 1` if `a == 2` and `b = 0` otherwise:
```
.bootstrap a b
001
```

3 bootstrapping:
  - `b = 1` if `a == 2` and `b = 0` otherwise
  - `c = 1` if `a == 0` and `c = 0` otherwise
  - `d = 0` for `a == 0`, `d = 1` for `a == 1` and _don't cares_ otherwise.

```
.bootstrap a b c d
001
100
01
```
