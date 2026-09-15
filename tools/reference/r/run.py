"""Run the isolated CRAN R oracle; no interpreter enters the chart runtime.

Usage: mise exec -- python3 tools/reference/r/run.py script.R [arguments...]
Set CHART_REFERENCE_R_HOME for an equivalent explicitly provisioned R installation.
CHART_REFERENCE_R_WORK and CHART_REFERENCE_R_LIBRARY can isolate generated resources.
"""
from pathlib import Path
import os
import subprocess
import shutil
import sys

ROOT = Path(__file__).resolve().parents[3]
local_r = shutil.which("Rscript")
if "CHART_REFERENCE_R_HOME" in os.environ:
    r_home = Path(os.environ["CHART_REFERENCE_R_HOME"]).resolve()
elif local_r:
    r_home = Path(subprocess.check_output([local_r, "--vanilla", "-e", "cat(R.home())"], text=True))
else:
    raise SystemExit("Install R 4.6.1 or set CHART_REFERENCE_R_HOME to its Resources directory.")
work = Path(os.environ.get("CHART_REFERENCE_R_WORK", ROOT / "target/reference-r")).resolve()
library = Path(os.environ.get("CHART_REFERENCE_R_LIBRARY", work / "library")).resolve()
work.mkdir(parents=True, exist_ok=True)
library.mkdir(parents=True, exist_ok=True)
font_config = work / "fonts.conf"
font_config.write_text('<?xml version="1.0"?><!DOCTYPE fontconfig SYSTEM "fonts.dtd"><fontconfig>'
                      f'<dir>{ROOT / "fixtures/capability/fonts"}</dir>'
                      f'<cachedir>{work / "font-cache"}</cachedir>'
                      '<alias><family>FinstackReference</family><prefer><family>Noto Sans</family></prefer></alias>'
                      '</fontconfig>')
x11 = ROOT / "target/reference-downloads/xquartz-expanded/XQuartzComponent.pkg/Payload/opt/X11/lib"
env = dict(os.environ, R_HOME=str(r_home), R_LIBS_USER=str(library),
           DYLD_LIBRARY_PATH=os.pathsep.join([str(r_home / "lib"), str(x11)]),
           FONTCONFIG_FILE=str(font_config), TZ="UTC", LC_ALL="C",
           RENV_PATHS_ROOT=str(work / "renv"),
           RENV_CONFIG_CACHE_ENABLED="FALSE", RENV_CONFIG_CONSENT="TRUE")
command = [str(r_home / "bin/exec/R"), "--vanilla", "--slave", "--file=" + str(Path(sys.argv[1]).resolve()), "--args", *sys.argv[2:]]
raise SystemExit(subprocess.call(command, cwd=ROOT, env=env))
