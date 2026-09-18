"""Independent rectangle oracle for the 11.1a crease instrument."""

import importlib.util
import pathlib
import unittest


CREASES = pathlib.Path(
    "_bmad-output/implementation-artifacts/11-1-signoff/creases.py"
)


def load_creases():
    spec = importlib.util.spec_from_file_location("creases", CREASES)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class CreasesTests(unittest.TestCase):
    def test_windows_remain_pinned_to_non_camp_rectangles(self):
        creases = load_creases()
        self.assertEqual(
            creases.WINDOWS,
            {
                "terrace-creases": (860, 190, 1060, 290),
                "open-snow-LL": (180, 620, 380, 700),
                "open-snow-LR": (950, 590, 1150, 670),
            },
        )
