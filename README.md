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

## Estimated results
| EPFL benchmarks        | AutoHoG   |                     |              |                   | Area-oriented technology mapping  |                     |              |                   | Multi-value-FBS-aware technology mapping   |                     |               |                  |
|:-----------------------|:----------|:--------------------|:-------------|:------------------|:----------------------------------|:--------------------|:-------------|:------------------|:-------------------------------------------|:--------------------|:--------------|:-----------------|
|                        | FBS size  | LinComb sqared norm | FBS cost     | Overall cost      | FBS size                          | LinComb sqared norm | FBS cost     | Overall cost      | FBS size                                   | LinComb sqared norm | FBS cost      | Overall cost     |
| Adder                  | 4         | 80                  | 40           | 20280             | 3                                 | 45                  | 38           | 4864              | 3                                          | 45                  | 38            | 4864             |
| Barrel shifter         | 4         | 80                  | 40           | 99840             | 4                                 | 80                  | 40           | 99840             | 4                                          | 80                  | 40            | 99840            |
| Divisor                | 4         | 80                  | 40           | 1150920           | 4                                 | 80                  | 40           | 523680            | 4                                          | 80                  | 40            | 523040           |
| Hypotenus              | 4         | 80                  | 40           | 4853000           | 4                                 | 80                  | 40           | 3164160           | 4                                          | 80                  | 40            | 3123040          |
| Log2                   | 4         | 80                  | 40           | 806400            | 4                                 | 80                  | 40           | 543040            | 4                                          | 80                  | 40            | 542920           |
| Max                    | 4         | 80                  | 40           | 86160             | 4                                 | 80                  | 40           | 82720             | 4                                          | 80                  | 40            | 82640            |
| Multiplier             | 4         | 80                  | 40           | 581200            | 4                                 | 80                  | 40           | 398280            | 4                                          | 80                  | 40            | 398280           |
| Sine                   | 4         | 336                 | 44           | 157432            | 4                                 | 336                 | 44           | 106392            | 4                                          | 336                 | 44            | 105512           |
| Square-root            | 4         | 80                  | 40           | 418080            | 4                                 | 80                  | 40           | 331040            | 4                                          | 80                  | 40            | 328720           |
| Sqaure                 | 4         | 80                  | 40           | 478720            | 4                                 | 80                  | 40           | 319280            | 4                                          | 80                  | 40            | 301880           |
| Round-robin arbiter    | 4         | 5                   | 40           | 457360            | 4                                 | 5                   | 40           | 464200            | 4                                          | 5                   | 40            | 464200           |
| Coding-cavlc           | 4         | 80                  | 40           | 22160             | 4                                 | 80                  | 40           | 21800             | 4                                          | 80                  | 40            | 21680            |
| ALU control unit       | 4         | 80                  | 40           | 3880              | 4                                 | 80                  | 40           | 3760              | 4                                          | 80                  | 40            | 3760             |
| Decoder                | 4         | 80                  | 40           | 11640             | 4                                 | 80                  | 40           | 11680             | 4                                          | 80                  | 40            | 11680            |
| I2C controller         | 4         | 80                  | 40           | 44360             | 4                                 | 80                  | 40           | 41280             | 4                                          | 80                  | 40            | 41160            |
| INT to FLOAT converter | 4         | 80                  | 40           | 7000              | 4                                 | 80                  | 40           | 6840              | 4                                          | 80                  | 40            | 6800             |
| Memory Controller      | 4         | 336                 | 44           | 1603624           | 4                                 | 336                 | 44           | 1562440           | 4                                          | 336                 | 44            | 1540704          |
| Priority encoder       | 4         | 80                  | 40           | 33320             | 4                                 | 80                  | 40           | 32720             | 4                                          | 80                  | 40            | 32720            |
| Lookahead XY router    | 4         | 80                  | 40           | 6960              | 4                                 | 80                  | 40           | 5360              | 4                                          | 80                  | 40            | 5040             |
| Voter                  | 4         | 336                 | 44           | 227480            | 4                                 | 336                 | 44           | 129888            | 4                                          | 80                  | 40            | 117440           |
| GEOMEAN                |           |                     |              | 115320.18         |                                   |                     |              | 87691.53          |                                           |                     |               | 86483.31         |