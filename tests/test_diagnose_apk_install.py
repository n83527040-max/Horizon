"""Static diagnostics must fail on signature verification errors."""
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
from zipfile import ZipFile

from tools.diagnose_apk_install import diagnose


class SignatureDiagnosticTests(unittest.TestCase):
    def test_a_failed_apksigner_cannot_be_reported_as_passing(self):
        def fake_run(args):
            if args[0] == 'aapt2':
                return 0, "package: name='com.example'\n"
            if args[args.index('--min-sdk-version') + 1] == '24':
                return 1, 'DOES NOT VERIFY\nERROR: invalid signature'
            return 0, 'Verifies\n'

        with tempfile.TemporaryDirectory() as temp:
            apk = Path(temp) / 'candidate.apk'
            with ZipFile(apk, 'w') as archive:
                for name in ('AndroidManifest.xml', 'classes.dex', 'resources.arsc', 'META-INF/TEST.RSA'):
                    archive.writestr(name, b'test')
            with patch('tools.diagnose_apk_install.run', side_effect=fake_run):
                report = diagnose(apk, 'aapt2', 'apksigner')
        self.assertFalse(report['passed'])
        self.assertFalse(report['apksignerMinSdk24']['verified'])
        self.assertIn('apksigner verification failed for minSdk 24', report['problems'])


if __name__ == '__main__':
    unittest.main()
