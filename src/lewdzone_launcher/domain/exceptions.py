"""Domain exceptions and error taxonomy (Rule 12).

| Code | Meaning                     |
| ---  | ---                         |
| 0    | success                     |
| 1    | runtime error               |
| 2    | usage error                 |
| 3    | network error               |
| 4    | download manager missing    |
| 5    | interrupted                 |
"""

from __future__ import annotations


class LewdzoneError(Exception):
    code = 1


class UsageError(LewdzoneError):
    code = 2


class NetworkError(LewdzoneError):
    code = 3


class DmMissingError(LewdzoneError):
    code = 4


class Interrupted(LewdzoneError):
    code = 5