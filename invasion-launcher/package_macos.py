#!/usr/bin/env python3
"""Build a portable, asset-free Apple silicon app and ZIP. Never launches a game."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import plistlib
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
VERSION = '0.9.0'

def run(*args, **kwargs):
    return subprocess.run(args, check=True, **kwargs)

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--binary',type=Path,default=ROOT/'target/play/iw4l')
    parser.add_argument('--output',type=Path,default=ROOT/'dist')
    args=parser.parse_args()
    output=args.output.resolve(); output.mkdir(parents=True,exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='invasion-release-') as folder:
        stage=Path(folder)
        app=stage/'Rust Minecraft Invasion.app'
        resources=app/'Contents/Resources'; macos=app/'Contents/MacOS'
        resources.mkdir(parents=True);macos.mkdir(parents=True)
        shutil.copyfile(args.binary,resources/'iw4l');(resources/'iw4l').chmod(0o755)
        run('codesign','--force','--sign','-','--identifier','org.rustinvasion.survival.game',str(resources/'iw4l'))
        licenses=resources/'Licenses';licenses.mkdir()
        for source,name in [('LICENSE','Apache-2.0.txt'),('NOTICE','NOTICE.txt'),('crates/ui/assets/OFL-Oxanium.txt','OFL-Oxanium.txt'),('crates/console/assets/COPYING-FreeFont.txt','COPYING-FreeFont.txt')]:
            shutil.copyfile(ROOT/source,licenses/name)
        metadata=json.loads(subprocess.check_output(['cargo','metadata','--locked','--offline','--filter-platform','aarch64-apple-darwin','--format-version','1'],cwd=ROOT,text=True))
        notices=[]
        for package in sorted(metadata['packages'],key=lambda p:(p['name'],p['version'])):
            if package.get('source') is None: continue
            directory=Path(package['manifest_path']).parent
            texts=[]
            for candidate in sorted(directory.iterdir()):
                if candidate.is_file() and candidate.name.upper().startswith(('LICENSE','LICENCE','COPYING','COPYRIGHT','NOTICE')):
                    try: texts.append(candidate.read_text())
                    except (UnicodeError,OSError): pass
            if package.get('license_file'):
                candidate=directory/package['license_file']
                if candidate.is_file():
                    value=candidate.read_text()
                    if value not in texts: texts.append(value)
            heading=package['name']+' '+package['version']+' — '+(package.get('license') or 'see package license')
            notices.append(heading+'\n'+(package.get('repository') or '')+'\n\n'+'\n\n'.join(texts))
        (licenses/'Rust-dependency-notices.txt').write_text('Bundled Rust dependency license notices\n\n'+'\n\n'+'\n\n'.join(notices))
        commit=subprocess.check_output(['git','-C',str(ROOT),'rev-parse','HEAD'],text=True).strip()
        (resources/'build.json').write_text(json.dumps({'version':VERSION,'source_commit':commit,'binary_sha256':hashlib.sha256((resources/'iw4l').read_bytes()).hexdigest(),'platform':'macOS Apple silicon'},indent=2)+'\n')
        with (app/'Contents/Info.plist').open('wb') as f:
            plistlib.dump({'CFBundleExecutable':'launch','CFBundleIdentifier':'org.rustinvasion.survival','CFBundleName':'Rust Minecraft Invasion','CFBundleDisplayName':'Rust Minecraft Invasion','CFBundlePackageType':'APPL','CFBundleVersion':'10','CFBundleShortVersionString':VERSION,'LSMinimumSystemVersion':'14.0','NSHighResolutionCapable':True,'NSDocumentsFolderUsageDescription':'Read the MW2 installation folder that you select.'},f)
        run('xcrun','clang','-mmacosx-version-min=14.0','-fobjc-arc','-framework','Cocoa',str(ROOT/'invasion-launcher/macos.m'),'-o',str(macos/'launch'))
        run('codesign','--force','--sign','-','--identifier','org.rustinvasion.survival',str(app))
        run('codesign','--verify','--strict',str(app))
        run(str(macos/'launch'),'--bundle-check')
        check=subprocess.run([str(resources/'iw4l'),'--help'],cwd=stage,capture_output=True,text=True,timeout=20)
        if check.returncode!=2 or 'usage: iw4l' not in check.stdout+check.stderr:raise RuntimeError('Game executable loader check failed')
        (stage/'README.txt').write_text('Rust Minecraft Invasion '+VERSION+'\n\nMove the app to Applications, then open it.\nChoose your owned Windows MW2 (2009) installation folder (English, multiplayer + base game files).\nClick Prepare Files to download Minecraft resources from Mojang.\nClick Open Game Menu when ready; select Create Game > Rust Invasion > Rust.\nChoose Intervention Quickscope and Equip.\nD-pad Right / 4 deploys the next earned killstreak.\n\nThis app is ad-hoc signed, not Apple-notarized. On a downloaded app, use macOS System Settings > Privacy & Security > Open Anyway if offered after trying to open it. No security settings or quarantine attributes need to be disabled.\n\nGame assets are not bundled. Source and license notices are in the app and public repository.\n')
        final=output/app.name
        if final.exists():
            info=plistlib.loads((final/'Contents/Info.plist').read_bytes())
            if info.get('CFBundleIdentifier')!='org.rustinvasion.survival':raise RuntimeError('Output app belongs to another project')
            shutil.rmtree(final)
        shutil.copytree(app,final)
        archive=output/f'Rust-Minecraft-Invasion-{VERSION}-macOS-arm64.zip'
        if archive.exists():archive.unlink()
        # Archive only the portable app and its guide; never package the runtime folder.
        with tempfile.TemporaryDirectory(prefix='invasion-zip-') as wrapper:
            wrapper=Path(wrapper);shutil.copytree(app,wrapper/app.name);shutil.copyfile(stage/'README.txt',wrapper/'README.txt')
            run('/usr/bin/ditto','-c','-k','--sequesterRsrc',str(wrapper),str(archive))
        digest=hashlib.sha256(archive.read_bytes()).hexdigest()
        (output/'SHA256SUMS.txt').write_text(f'{digest}  {archive.name}\n')
        print('Packaged',archive,'SHA256',digest)
        print('No game, renderer or audio playback started.')

if __name__=='__main__':main()
