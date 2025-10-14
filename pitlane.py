import click
from halo import Halo
from pyfiglet import Figlet
import os
import shutil
import subprocess

# Modeled after IDF-esp tools

# ASCII Start
f = Figlet(font="slant")
# Halo spinner
spinner = Halo(text="Loading", spinner="dots")

""" build: builds and generates a debug or release elf file
    build_rs: build a release version of the program without debug features
    flash: upload the elf file on the STM platform
    debug: launches the debugger and connects to the STM platform
    service: starts up the service CLI 
"""


@click.group()
def cli():
    pass


@click.command()
@click.option("--release/--debug", default=False, help="Build type")
def build(release):
    click.echo("Build Mode")
    # Check toolchain availability
    spinner = Halo("Checking Toolchain", spinner="dots")
    spinner.start()
    for tool in ["cmake", "arm-none-eabi-gcc"]:
        if not shutil.which(tool):
            print(f"[-] {tool} is not installed or not in PATH")
            print("[-] Stopping")
            return
    spinner.stop()

    try:
        spinner = Halo("Building Debug mode", spinner="dots")
        spinner.start()

        preset = "Release" if release else "Debug"
        cp = subprocess.run(
            ["cmake", "--preset", preset], capture_output=True, text=True, check=True
        )
        click.echo(cp.stdout.strip())
    except subprocess.CalledProcessError as e:
        spinner.stop()
        print(f"[-] Error configuring {e.stderr.strip()}")


@click.option("--flash", help="upload the elf file on the STM platform")
def flash():
    pass


if __name__ == "__main__":
    print(f.renderText("PULSAR"))
    cli()
