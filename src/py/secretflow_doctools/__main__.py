import click

from secretflow_doctools import cmd
from secretflow_doctools.l10n import gettext as _
from secretflow_doctools.utils.logging import configure_logging


@click.group()
def cli():
    configure_logging()


_NO_WRAP = "\b"
_NEWLINE = "\n\n\b"

opt_config_dir = click.option(
    "-c",
    "--config-dir",
    required=False,
    type=click.Path(exists=True, file_okay=False),
    help=_("path to the directory containing Sphinx's `conf.py` file") + _NEWLINE,
)

opt_source_dir = click.option(
    "-i",
    "--source-dir",
    required=False,
    type=click.Path(exists=True, file_okay=False),
    help=_(
        "path to the directory containing documentation source files."
        " generated page paths will be relative to this directory"
    )
    + _NEWLINE,
)

opt_output_dir = click.option(
    "-o",
    "--output-dir",
    required=False,
    type=click.Path(file_okay=False),
    help=_("path for the generated output files") + _NEWLINE,
)


@cli.command(help=_("build docs"))
@opt_config_dir
@opt_source_dir
@opt_output_dir
@click.option(
    "-l",
    "--lang",
    default=(),
    multiple=True,
    help=_(
        """
        the language to build docs in. to build in multiple languages
        at once, specify this option multiple times, for example:

            --lang en --lang zh_CN

        {_NO_WRAP}
        see https://www.sphinx-doc.org/en/master/usage/configuration.html#confval-language
        {_NO_WRAP}
        """
    ).format(_NO_WRAP=_NO_WRAP),
)
@click.option(
    "--mapped-path",
    multiple=True,
    help=_(
        """
        path mapping, in the format of:

        {_NO_WRAP}
            actual-repo:actual-path:mapped-repo:mapped-path
        {_NO_WRAP}
        """
    ).format(_NO_WRAP=_NO_WRAP),
)
@click.argument("sphinx-args", nargs=-1)
def build(*args, **kwargs):
    cmd.build(*args, **kwargs)


@cli.command(help=_("locally preview previously built docs"))
@opt_config_dir
@opt_source_dir
@opt_output_dir
@click.argument("flask-args", nargs=-1)
def preview(*args, **kwargs):
    cmd.preview(*args, **kwargs)


@cli.command(help=_("update translation files"))
@opt_config_dir
@opt_source_dir
@opt_output_dir
@click.option(
    "-l",
    "--lang",
    required=True,
    help=_("the language to be translated"),
)
@click.option(
    "--with-swagger",
    is_flag=True,
    help=_(
        "in addition to updating .po files, also update translation files"
        " for translating Swagger schemas"
    )
    + _NEWLINE,
)
@click.argument("sphinx-args", nargs=-1)
def update_translations(*args, **kwargs):
    cmd.update_translations(*args, **kwargs)


@cli.command(hidden=True, help=_("publish built docs to npm"))
@click.option("--name", required=True, help=_("full name of the npm package"))
@click.option("--tag", default=None, help=_("the dist-tag to use"))
@click.option(
    "--index-js",
    required=True,
    type=click.Path(exists=True, dir_okay=False),
    help=_("path to `dist/index.js`"),
)
@click.option(
    "--registry",
    default="https://registry.npmjs.org",
    help=_("the npm registry to publish the package to"),
)
def publish(*args, **kwargs):
    cmd.publish(*args, **kwargs)


@cli.command(help=_("clean up files from previously built docs"))
@opt_config_dir
@opt_source_dir
@opt_output_dir
def clean(*args, **kwargs):
    cmd.clean(*args, **kwargs)


if __name__ == "__main__":
    cli()
