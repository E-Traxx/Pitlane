#!/usr/bin/env python3
# Simple helper for building and flashing Pulsar or NOVA firmware.

from __future__ import annotations

import shutil
import subprocess
from pathlib import Path
from pyfiglet import Figlet
from halo import Halo
import click


ROOT = Path(__file__).resolve().parents[2]
DEFAULT_TARGET = "Pulsar.elf"
DEFAULT_PRESET = "Debug"
DEFAULT_PROGRAMMER = "STM32_Programmer_CLI"

figlet = Figlet(font="slant")


def ensure_tools(tools: list[str]) -> None:
    # Abort the command if a required tool cannot be found.

    missing = [tool for tool in tools if shutil.which(tool) is None]
    if missing:
        for tool in missing:
            click.echo(f"[-] {tool} is not installed or not available on PATH")
        raise click.Abort()


def configure_and_build(preset: str, project_root: Path) -> None:
    # Run the standard CMake configure and build steps.

    spinner = Halo(text=f"Configuring ({preset})", spinner="dots")
    spinner.start()
    try:
        subprocess.check_call(["cmake", "--preset", preset], cwd=project_root)
    except subprocess.CalledProcessError as exc:
        spinner.fail("Configuration failed")
        raise click.ClickException(
            f"cmake --preset {preset} exited with {exc.returncode}"
        ) from exc
    spinner.succeed("Configuration complete")

    spinner = Halo(text=f"Building ({preset})", spinner="dots")
    spinner.start()
    try:
        subprocess.check_call(["cmake", "--build", "--preset", preset], cwd=project_root)
    except subprocess.CalledProcessError as exc:
        spinner.fail("Build failed")
        raise click.ClickException(
            f"Build for preset {preset} exited with {exc.returncode}"
        ) from exc
    spinner.succeed("Build finished")


def resolve_image_path(preset: str, image: Path | None) -> Path:
    # Return the firmware image to flash.

    if image is None:
        image = ROOT / "build" / preset / DEFAULT_TARGET
    if not image.exists():
        raise click.ClickException(f"Firmware image not found at {image}")
    return image


@click.group(help="E-Traxx ECU Build & Flash Tool.")
def cli() -> None:
    # Entry point for the Pitlane CLI interface.

    click.echo(figlet.renderText("PITLANE"))


@cli.command(help="Configure and compile the firmware.")
@click.option(
    "--release/--debug",
    default=False,
    show_default=True,
    help="Select the CMake preset",
)
@click.option(
    "--root",
    type=click.Path(path_type=Path),
    default=None,
    help="Project root directory (defaults to repository root)",
)
@click.argument(
    "root_arg",
    type=click.Path(path_type=Path),
    required=False,
)
def build(release: bool, root: Path | None, root_arg: Path | None) -> None:
    # Configure and compile the firmware.

    if root is not None and root_arg is not None:
        raise click.ClickException("Use either the --root option or positional root argument, not both.")

    preset = "Release" if release else DEFAULT_PRESET
    ensure_tools(["cmake", "arm-none-eabi-gcc"])

    selected_root = root if root is not None else root_arg
    project_root = selected_root.resolve() if selected_root is not None else ROOT
    if not project_root.is_dir():
        raise click.ClickException(f"Root directory not found: {project_root}")
    configure_and_build(preset, project_root)


@cli.command(help="Build the project (if needed) and flash it with STM32CubeProgrammer.")
@click.option(
    "--release/--debug",
    default=False,
    show_default=True,
    help="Flash the release build",
)
@click.option(
    "--image",
    type=click.Path(path_type=Path),
    default=None,
    help="Optional path to a .elf/.bin file",
)
@click.option(
    "--address", default=None, help="Optional address when flashing raw binaries"
)
@click.option("--no-verify", is_flag=True, help="Skip verification after flashing")
@click.option("--no-reset", is_flag=True, help="Skip reset after flashing")
def flash(
    release: bool,
    image: Path | None,
    address: str | None,
    no_verify: bool,
    no_reset: bool,
) -> None:
    # Build the project (if needed) and flash it with STM32CubeProgrammer.

    ensure_tools(["cmake", DEFAULT_PROGRAMMER])

    preset = "Release" if release else DEFAULT_PRESET
    configure_and_build(preset, ROOT)

    firmware = resolve_image_path(preset, image)

    cmd = [DEFAULT_PROGRAMMER, "-c", "port=SWD", "freq=4000", "-d", str(firmware)]
    if address:
        cmd.append(address)
    if not no_verify:
        cmd.append("-v")
    if not no_reset:
        cmd.append("-rst")

    spinner = Halo(text="Flashing device", spinner="dots")
    spinner.start()
    try:
        subprocess.check_call(cmd)
    except subprocess.CalledProcessError as exc:
        spinner.fail("Flashing failed")
        raise click.ClickException(
            f"STM32CubeProgrammer exited with {exc.returncode}"
        ) from exc
    spinner.succeed("Flashing complete")


@cli.command(help="Show connected programmers using STM32CubeProgrammer.")
def devices() -> None:
    # Show connected programmers using STM32CubeProgrammer.

    ensure_tools([DEFAULT_PROGRAMMER])
    spinner = Halo(text="Checking connected devices", spinner="dots")
    spinner.start()
    try:
        subprocess.check_call([DEFAULT_PROGRAMMER, "-l"])
    except subprocess.CalledProcessError as exc:
        spinner.fail("Could not list devices")
        raise click.ClickException("Listing devices failed") from exc
    spinner.succeed("Device check complete")


if __name__ == "__main__":
    cli()
