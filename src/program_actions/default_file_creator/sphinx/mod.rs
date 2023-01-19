pub const DEFAULT_SPHINX_CONF_PY_IN_CONTENTS: &'static str =
"# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = \"@PROJECT_NAME@\"
copyright = '2023, @PROJECT_VENDOR@'
author = \"@PROJECT_VENDOR@\"
release = \"@PROJECT_VERSION@\"

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

# templates_path = ['_templates']
# exclude_patterns = []

primary_domain = 'cpp'
highlight_language = 'cpp'

# Breathe (https://breathe.readthedocs.io/en/latest/) is the bridge between Sphinx and Doxygen. It
# allows Sphinx to use Doxygen's output for its own method of documentation generation.
# Exhale (https://exhale.readthedocs.io/en/latest/) uses 'Breathe' and Doxygen's output to
# automatically generate a full API documentation hierarchy for your project.
# For C/C++ documentation, Breathe is required, but Exhale is not.

extensions = [
  # pip install breathe
  \"breathe\",
  # pip install exhale
  \"exhale\"
]

breathe_default_project = \"@PROJECT_NAME@\"

exhale_args = {
  # The \"containment folder\" is required to be somewhere inside docs/, which is annoying.
  # Just remember to add docs/api to .gitignore.
  \"containmentFolder\": \"./api\",
  \"rootFileName\": \"lib_root.rst\",
  \"rootFileTitle\": \"Library API\",
  \"doxygenStripFromPath\": \"..\",
  \"createTreeView\": True,
  \"exhaleExecutesDoxygen\": False
}

# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

# html_theme = 'alabaster'

# pip install sphinx_rtd_theme
html_theme = 'sphinx_rtd_theme'
# html_static_path = ['_static']";

pub const DEFAULT_SPHINX_INDEX_RST_CONTENTS: &'static str =
"Welcome to (Your project name)'s Documentation!
==============================================

.. toctree::
  :maxdepth: 2
  :caption: Contents:

  .. When uncommented, this would like to the lib_root.rst file generated
  .. by Exhale if you're using it in Sphinx's conf.py.in file.
  api/lib_root

Indices and tables
==================

* :ref:`genindex`
* :ref:`modindex`
* :ref:`search`";

