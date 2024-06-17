class LbfParser:
    def _strip_comments(self, line):
        # strip comments and spaces
        if (idx:= line.find("#")) >= 0:
            line = line[:idx]
        line = line.strip()
        return line

    def _get_callback(self, callback):
        def _empty_callback(*args):
            pass

        if callback is None:
            return _empty_callback
        else:
            return callback

    def __init__(self,
                 inputs_callback=None,
                 outputs_callback=None,
                 params_callback=None,
                 lincomb_callback=None,
                 const_callback=None,
                 bootstrap_callback=None,
                 end_callback=None):
        self.inputs_callback = self._get_callback(inputs_callback)
        self.outputs_callback = self._get_callback(outputs_callback)
        self.params_callback = self._get_callback(params_callback)
        self.lincomb_callback = self._get_callback(lincomb_callback)
        self.const_callback = self._get_callback(const_callback)
        self.bootstrap_callback = self._get_callback(bootstrap_callback)
        self.end_callback = self._get_callback(end_callback)

    def parse(self, lines):
        idx = 0
        while idx < len(lines):
            line, idx = self._strip_comments(lines[idx]), idx + 1
            match line.split():
                case[".inputs", *inps]:
                    self.inputs_callback(inps)  # TODO: support multi-line

                case[".outputs", *outs]:
                    self.outputs_callback(outs)  # TODO: support multi-line

                case[".params", *params]:
                    self.params_callback(params)

                case[".lincomb", *inps, out]:
                    line, idx = self._strip_comments(lines[idx]), idx + 1
                    coefs = list(map(int, line.split()))

                    const_coef = 0
                    if len(coefs) > len(inps):
                        *coefs, const_coef = coefs
                    assert(len(coefs) == len(inps)
                           ), "lincomb input and coefficient count missmatch"

                    if len(inps) > 0:
                        self.lincomb_callback(out, inps, coefs, const_coef)
                    else:
                        self.const_callback(out, const_coef)

                case[".bootstrap", inp, *outs]:
                    assert(len(outs) > 0), "bootstrap expected at least"
                    " one output"
                    tables = list()
                    for _ in range(len(outs)):
                        line, idx = self._strip_comments(lines[idx]), idx + 1
                        tables.append(list(map(int, line)))
                    self.bootstrap_callback(outs, inp, tables)

                case[".end"]:
                    self.end_callback()

                case _:
                    assert(False), f"cannot parse line no {idx}: '{line}'"


if __name__ == '__main__':
    def inputs_callback(*args):
        print(f"inputs_callback {args}")

    def outputs_callback(*args):
        print(f"outputs_callback {args}")

    def params_callback(*args):
        print(f"params_callback {args}")

    def lincomb_callback(*args):
        print(f"lincomb_callback {args}")

    def const_callback(*args):
        print(f"const_callback {args}")

    def bootstrap_callback(*args):
        print(f"bootstrap_callback {args}")

    def end_callback(*args):
        print(f"end_callback {args}")

    parser = LbfParser(
        inputs_callback=inputs_callback,
        outputs_callback=outputs_callback,
        params_callback=params_callback,
        lincomb_callback=lincomb_callback,
        const_callback=const_callback,
        bootstrap_callback=bootstrap_callback,
        end_callback=end_callback)

    with open("sample.lbf", "r") as fs:
        lines = fs.readlines()
        parser.parse(lines)
