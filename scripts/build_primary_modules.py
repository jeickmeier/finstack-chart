"""One owner for building/copying the actual primary proof adapters."""
import shutil
import sys

WASM_ADAPTER_FILES = ('authoring.cjs', 'authoring.d.cts', 'interpolation.cjs', 'interpolation.d.cts', 'scales.cjs', 'scales.d.cts', 'examples.cjs', 'examples.d.cts', 'hierarchy.cjs', 'hierarchy.d.cts')


def python_module(root, output, target, run):
    run('python-primary-build', ['cargo', 'build', '-p', 'chart-python', '--features', 'extension-module,extension-proof', '--locked'])
    module = output / 'python-module'
    module.mkdir(exist_ok=True)
    library = target / 'debug' / ('libchart_python.dylib' if sys.platform == 'darwin' else 'libchart_python.so')
    shutil.copy2(library, module / 'chart_python.so')
    return module


def wasm_module(root, output, target, run, cli):
    run('wasm-primary-build', ['cargo', 'build', '-p', 'chart-wasm', '--features', 'extension-proof', '--target', 'wasm32-unknown-unknown', '--locked'])
    module = output / 'wasm-module'
    run('wasm-primary-generate', [cli, target / 'wasm32-unknown-unknown/debug/chart_wasm.wasm', '--target', 'nodejs', '--out-dir', module])
    for name in WASM_ADAPTER_FILES:
        shutil.copy2(root / 'packages/wasm' / name, module / name)
    return module
