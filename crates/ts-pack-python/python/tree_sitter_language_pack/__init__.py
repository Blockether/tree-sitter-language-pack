from typing import TypeAlias

from tree_sitter_language_pack._native import (
    LanguageNotFoundError,
    available_languages,
    get_binding,
    get_language,
    get_parser,
    has_language,
)

SupportedLanguage: TypeAlias = str

__all__ = [
    "LanguageNotFoundError",
    "SupportedLanguage",
    "available_languages",
    "get_binding",
    "get_language",
    "get_parser",
    "has_language",
]
