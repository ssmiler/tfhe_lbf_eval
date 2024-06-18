from parser import LbfParser
from enum import Enum
from typing import List, Dict, TextIO


class LbfCircuit:
    class Node:
        def __init__(self, name):
            self.name = name

    class Input(Node):
        def __init__(self, out):
            super().__init__(out)
            self.out = out

    class Const(Node):
        def __init__(self, out, val):
            super().__init__(out)
            self.out = out
            self.val = val

    class Lincomb(Node):
        def __init__(self,
                     out: str,
                     inps: List[str],
                     coefs: List[int],
                     const_coef=int):
            super().__init__(out)
            self.out = out
            self.inps = inps
            self.coefs = coefs
            self.const_coef = const_coef

    class Bootstrap(Node):
        def __init__(self,
                     outs: List[str],
                     inp: str,
                     tables: List[List[int]]):
            super().__init__(",".join(outs))
            self.outs = outs
            self.inp = inp
            self.tables = tables

    def __init__(self):
        self.node_ids = list()      # node ids in topological order
        self.out_ids = list()       # node ids which are circuit output
        self.successors = dict()    # predecessors ids by node id
        self.predecessors = dict()  # successors ids by node id
        self.node_by_id = dict()    # Node object by node id

    def _add_edge(self, u, v):
        self.predecessors[v].append(u)
        self.successors[u].append(v)

    def _add_node(self, node: Node):
        self.node_by_id[node.name] = node
        self.node_ids.append(node.name)
        self.predecessors[node.name] = list()
        self.successors[node.name] = list()

    def add_input(self, out: str):
        self._add_node(LbfCircuit.Input(out))

    def add_const(self, out: str, val: int):
        self._add_node(LbfCircuit.Const(out, val))

    def add_lincomb(self,
                    out: str,
                    inps: List[str],
                    coefs: List[int],
                    const_coef: int = 0):
        node = LbfCircuit.Lincomb(out, inps, coefs, const_coef)
        self._add_node(node)
        for inp in inps:
            self._add_edge(inp, node.name)

    def add_bootstrap(self,
                      outs: List[str],
                      inp: str,
                      tables: List[List[int]]):
        node = LbfCircuit.Bootstrap(outs, inp, tables)
        self._add_node(node)
        self._add_edge(inp, node.name)

    def add_output(self, out: str):
        self.out_ids.append(out)

    def finalize(self):
        pass


class LbfCircuitParser:
    def __init__(self):
        pass

    def parse_file(filename: str):
        with open(filename, "r") as fs:
            return LbfCircuitParser.parse_stream(fs)

    def parse_stream(fs: TextIO):
        lines = fs.readlines()
        return LbfCircuitParser.parse_lines(lines)

    def parse_lines(lines: List[str]):
        circuit = LbfCircuit()

        def inputs_callback(inps):
            for inp in inps:
                circuit.add_input(inp)

        def outputs_callback(outs):
            for out in outs:
                circuit.add_output(out)

        def lincomb_callback(out, inps, coefs, const_coef):
            circuit.add_lincomb(out, inps, coefs, const_coef)

        def const_callback(out, val):
            circuit.add_const(out, val)

        def bootstrap_callback(outs, inp, tables):
            circuit.add_bootstrap(outs, inp, tables)

        def end_callback(*args):
            circuit.finalize()

        parser = LbfParser(
            inputs_callback=inputs_callback,
            outputs_callback=outputs_callback,
            lincomb_callback=lincomb_callback,
            const_callback=const_callback,
            bootstrap_callback=bootstrap_callback,
            end_callback=end_callback)

        parser.parse(lines)

        return circuit


if __name__ == '__main__':
    circuit = LbfCircuitParser.parse_file("sample.lbf")

    print("LBF circuit inputs:")
    for name in circuit.node_ids:
        node = circuit.node_by_id[name]
        match node:
            case LbfCircuit.Input(out=out):
                print(f"\tinput({out})")
            case LbfCircuit.Const(out=out, val=val):
                print(f"\t{out} = const({val})")

    print("LBF circuit node_ids:")
    for name in circuit.node_ids:
        node = circuit.node_by_id[name]
        match node:
            case LbfCircuit.Lincomb(out=out,
                                    inps=inps,
                                    coefs=coefs,
                                    const_coef=const_coef):
                expr = f"{const_coef}"
                for inp, coef in zip(inps, coefs):
                    expr += f" + {coef}" if coef >= 0 else f" - {-coef}"
                    expr += f" * {inp}"

                print(f"\t{out} = {expr}")
            case LbfCircuit.Bootstrap(outs=outs, inp=inp, tables=tables):
                out_str = ", ".join(outs)
                tables_str = ", ".join(map(str, tables))
                print(f"\t{out_str} = LUT({inp}, {tables_str})")

    print("LBF circuit outputs:")
    for out in circuit.out_ids:
        print(f"\toutput({out})")
