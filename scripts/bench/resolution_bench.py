#!/usr/bin/env python3
"""Offline geometry and simulation-resolution measurements for story 10.6.

The benchmark intentionally uses only the Python standard library.  Its detail rule is a
measurement stand-in, not game art: it gives an exposed sub-cell surface a small seeded height
variation so the greedy mesher has real fine geometry to account for.
"""


def detail_offset(seed, x, y, z):
    """Return a deterministic exposed-surface displacement in fine voxels.

    // NOTE: This is a measurement stand-in for 10.4's authored terrain look, not a visual
    decision.  The small value-noise displacement deliberately breaks flat greedy runs.
    """
    value = seed ^ (x * 0x9E3779B1) ^ (y * 0x85EBCA77) ^ (z * 0xC2B2AE3D)
    value ^= value >> 16
    value = (value * 0x7FEB352D) & 0xFFFFFFFF
    value ^= value >> 15
    return value % 5 - 2
