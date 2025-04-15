import argparse
import numpy as np
import subprocess

from circuit import LbfCircuit, LbfCircuitParser


OPTIMIZER_DEFAULT_PATH = "./concrete/compilers/concrete-optimizer/optimizer"


def find_min_fbs_params(circuit: LbfCircuit, strict_fbs_size):
    min_fbs_size = 0
    min_sq_norm2 = 0
    for name in circuit.node_ids:
        node = circuit.node_by_id[name]
        match node:

            case LbfCircuit.Bootstrap(tables=tables, inp=inp):
                min_fbs_size_local = min_fbs_size
                if strict_fbs_size:
                    size = max(map(len, tables))
                    min_fbs_size_local = max(min_fbs_size_local, size)
                else:
                    for table in tables:
                        min_fbs_size_local = max(
                            min_fbs_size_local, len(table)//2)
                        while min_fbs_size_local < len(table):
                            delta = len(table) - min_fbs_size_local
                            p1 = np.array(table[-delta:])
                            p2 = np.array(table[:delta])
                            if np.all(p1 != p2):
                                break
                            min_fbs_size_local += 1
                min_fbs_size = max(min_fbs_size_local, min_fbs_size)

                match circuit.node_by_id[inp]:
                    case LbfCircuit.Lincomb(coefs=coefs):
                        sq_norm2 = sum(map(lambda coef: coef*coef, coefs))
                    case _:
                        assert(False), "expected lincomb as bootstrapping input"

                # account for multi-output FBS
                if len(tables) > 1:
                    # sq_norm2 *= min_fbs_size_local * min_fbs_size_local # original CIM19 noise estimate
                    sq_norm2 *= min_fbs_size_local # optimized noise from GBA21

                min_sq_norm2 = max(sq_norm2, min_sq_norm2)

    return min_fbs_size, min_sq_norm2


def find_circuit_params(circuit: LbfCircuit, old_format=False):
    nb_bootstrappings = 0
    node_depth = dict()
    for name in circuit.node_ids:
        node = circuit.node_by_id[name]
        match node:
            case LbfCircuit.Input(name=name):
                node_depth[name] = 0
            case LbfCircuit.Const(name=name):
                node_depth[name] = 0
            case LbfCircuit.Lincomb(name=name, inps=inps):
                node_depth[name] = max(map(lambda inp: node_depth[inp], inps))
            case LbfCircuit.Bootstrap(name=name, outs=outs):
                d = 1 + max(map(lambda inp: node_depth[inp], inps))
                if old_format:
                    node_depth[name] = d
                else:
                    for out in outs:
                        node_depth[out] = d
                nb_bootstrappings += 1
    return nb_bootstrappings, max(node_depth.values())


def estimated_boot_cost(fbs_size, sq_norm2, opt_path):
    output = subprocess.check_output([opt_path, f"--precision={fbs_size}", f"--sq-norm2={sq_norm2}"], stderr=subprocess.DEVNULL)
    return int(output.decode().split(",")[-2].strip())


if __name__ == '__main__':
    parser = argparse.ArgumentParser(
        description="Compute/estimate LBF circuits",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    parser.add_argument("filename", help="input lbf filename")

    group = parser.add_argument_group(
        "functional bootstrapping (fbs) parameters")
    group.add_argument("--fbs_size", type=int, help="fbs size,"
                       " find from lbf circuit if none")
    group.add_argument("--sq_norm2", type=int, help="lincomb max norm2 squared,"
                       " find from lbf circuit if none")
    group.add_argument("--strict_fbs_size", action="store_true",
                       help="do not use anti-cyclic ring property")

    group = parser.add_argument_group("circuit evaluation configuration")
    group.add_argument("--nb_cores", type=int, default=1,
                       help="number of execution cores")

    parser.add_argument("--opt_path",
                        default=OPTIMIZER_DEFAULT_PATH,
                        help="path to patched concrete v0-optimizer")

    parser.add_argument("-b", action="store_true",
                        help="use deprectated lbf format with merged MVFBS outputs")

    args = parser.parse_args()

    circuit = LbfCircuitParser.parse_file(args.filename, args.b)

    min_fbs_size, min_sq_norm2 = find_min_fbs_params(
        circuit, args.strict_fbs_size)

    assert(args.fbs_size is None or args.fbs_size >= min_fbs_size)
    assert(args.sq_norm2 is None or args.sq_norm2 >= min_sq_norm2)

    fbs_size = min_fbs_size if args.fbs_size is None else args.fbs_size
    sq_norm2 = min_sq_norm2 if args.sq_norm2 is None else args.sq_norm2

    print("Evaluation fbs parameters (and the lbf circuit minimal ones):")
    print(f"\tfbs size: {fbs_size} ({min_fbs_size})")
    print(f"\tlincomb squared norm2: {sq_norm2} ({min_sq_norm2})")

    bootstrap_cost = estimated_boot_cost(fbs_size, sq_norm2, args.opt_path)
    print(f"Estimated bootstrapping cost: {bootstrap_cost}")

    nb_bootstrappings, depth = find_circuit_params(circuit, args.b)
    print(f"Number of bootstrappings: {nb_bootstrappings}")

    total_cost = bootstrap_cost * \
        max(int(np.ceil(nb_bootstrappings / args.nb_cores)), depth)
    print(f"Estimated circuit cost: {total_cost}")
