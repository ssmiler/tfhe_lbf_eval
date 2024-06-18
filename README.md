# Execution cost estimator for LBF circuits
Estimate execution cost of a LBF (Lincomb Bootstrap Format) files using concrete estimation tools

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
