# SPDX-License-Identifier: AGPL-3.0-only
"""Shared fixtures."""

from __future__ import annotations

import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
