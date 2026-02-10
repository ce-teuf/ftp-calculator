#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Script de release automatisé pour FTP_CALCULATOR
Gère le versioning, la création de tags, et la préparation des artefacts
"""

import argparse
import subprocess
import sys
import os
import re
from pathlib import Path
from datetime import datetime
import toml

# Force UTF-8 encoding for Windows
if sys.platform == 'win32':
    import codecs
    sys.stdout = codecs.getwriter('utf-8')(sys.stdout.buffer, 'strict')
    sys.stderr = codecs.getwriter('utf-8')(sys.stderr.buffer, 'strict')

class ReleaseManager:
    def __init__(self):
        self.project_root = Path(__file__).parent.parent
        self.cargo_toml = self.project_root / "Cargo.toml"
        self.pyproject_toml = self.project_root / "python-lib" / "pyproject.toml"

    def run_command(self, cmd, cwd=None):
        """Exécute une commande shell"""
        cwd = cwd or self.project_root
        print(f"🤖 Exécution: {cmd}")
        result = subprocess.run(cmd, shell=True, cwd=cwd, capture_output=True, text=True)
        if result.returncode != 0:
            print(f"❌ Erreur: {result.stderr}")
            return False
        print(f"✅ Succès: {result.stdout}")
        return True

    def get_current_version(self):
        """Récupère la version actuelle depuis Cargo.toml"""
        with open(self.cargo_toml, 'r') as f:
            cargo_data = toml.load(f)

        # Cherche la version dans les membres du workspace
        for member in cargo_data['workspace']['members']:
            member_toml = self.project_root / member / "Cargo.toml"
            if member_toml.exists():
                with open(member_toml, 'r') as f:
                    member_data = toml.load(f)
                    if 'package' in member_data and 'version' in member_data['package']:
                        return member_data['package']['version']

        raise ValueError("Version non trouvée dans les Cargo.toml")

    def update_version(self, new_version):
        """Met à jour la version dans tous les fichiers de configuration"""
        print(f"📦 Mise à jour de la version vers {new_version}")

        # Met à jour tous les Cargo.toml des crates
        for member in ["ftp_core", "ftp_core_bindings_c", "ftp_core_bindings_pyo3"]:
            crate_toml = self.project_root / "crates" / member / "Cargo.toml"
            if crate_toml.exists():
                with open(crate_toml, 'r') as f:
                    data = toml.load(f)

                if 'package' in data:
                    data['package']['version'] = new_version

                with open(crate_toml, 'w') as f:
                    toml.dump(data, f)
                print(f"✅ {crate_toml} mis à jour")

        # Met à jour pyproject.toml Python
        if self.pyproject_toml.exists():
            with open(self.pyproject_toml, 'r') as f:
                data = toml.load(f)

            if 'project' in data:
                data['project']['version'] = new_version
            elif 'tool' in data and 'poetry' in data['tool']:
                data['tool']['poetry']['version'] = new_version

            with open(self.pyproject_toml, 'w') as f:
                toml.dump(data, f)
            print(f"✅ {self.pyproject_toml} mis à jour")

    def validate_release_readiness(self):
        """Valide que le projet est prêt pour une release"""
        print("🔍 Validation de l'état du projet...")

        checks = [
            ("Formatage", self.run_command("cargo fmt --all -- --check")),
            ("Linting", self.run_command("cargo clippy --workspace -- -D warnings")),
            ("Build Rust", self.run_command("cargo build --release")),
            ("Build Python", self.run_command("make build-py-bindings")),
        ]

        all_ok = all(success for _, success in checks)

        if not all_ok:
            print("❌ Le projet n'est pas prêt pour la release")
            return False

        print("✅ Toutes les validations sont passées")
        return True

    def create_git_tag(self, version, message=None):
        """Crée un tag Git pour la release"""
        message = message or f"Release v{version}"

        if not self.run_command(f'git tag -a "v{version}" -m "{message}"'):
            return False

        if not self.run_command("git push --tags"):
            return False

        print(f"✅ Tag v{version} créé et poussé")
        return True

    def generate_changelog(self, version):
        """Génère un changelog basique (à améliorer selon les besoins)"""
        changelog_file = self.project_root / "CHANGELOG.md"

        # Récupère les commits depuis le dernier tag
        result = subprocess.run(
            "git log --oneline --no-decorate",
            shell=True, capture_output=True, text=True
        )

        commits = result.stdout.split('\n')[:10]  # 10 derniers commits

        with open(changelog_file, 'r+') as f:
            content = f.read()
            f.seek(0, 0)

            commits_text = ''.join(f'- {commit}\n' for commit in commits if commit)
            changelog_entry = f"""## v{version} - {datetime.now().strftime('%Y-%m-%d')}

### Nouvelles fonctionnalités
- À compléter

### Corrections
- À compléter

### Modifications
{commits_text}
"""
            f.write(changelog_entry + content)

        print("✅ Changelog mis à jour")

    def build_release_artifacts(self):
        """Construit tous les artefacts de release"""
        print("🏗️  Construction des artefacts de release...")

        artifacts = [
            ("Bindings C", "make build-c-bindings"),
            ("Bindings Python", "make build-py-bindings"),
            ("Documentation", "make build-docs"),
        ]

        for name, cmd in artifacts:
            print(f"📦 Construction: {name}")
            if not self.run_command(cmd):
                print(f"❌ Échec de la construction: {name}")
                return False

        print("✅ Tous les artefacts construits")
        return True

    def create_release(self, version, bump_type="patch", dry_run=False):
        """Crée une nouvelle release"""
        print(f"🚀 Lancement de la release v{version}")

        if dry_run:
            print("🔶 MODE SIMULATION - Aucune modification ne sera effectuée")

        # 1. Validation
        if not self.validate_release_readiness():
            return False

        # 2. Mise à jour des versions
        if not dry_run:
            self.update_version(version)

        # 3. Construction des artefacts
        if not self.build_release_artifacts():
            return False

        # 4. Génération du changelog
        if not dry_run:
            self.generate_changelog(version)

        # 5. Commit des modifications
        if not dry_run:
            if not self.run_command('git add .'):
                return False
            if not self.run_command(f'git commit -m "Release v{version}"'):
                return False

        # 6. Création du tag
        if not dry_run:
            if not self.create_git_tag(version):
                return False

        print(f"🎉 Release v{version} créée avec succès!")

        # Instructions pour la suite
        print("\n📋 Prochaines étapes:")
        print("1. Les workflows GitHub vont déclencher la publication automatique")
        print("2. Vérifier les artefacts sur GitHub Releases")
        print("3. Mettre à jour la documentation si nécessaire")

        return True

    def bump_version(self, current_version, bump_type):
        """Incrémente la version selon le type demandé"""
        major, minor, patch = map(int, current_version.split('.'))

        if bump_type == "major":
            major += 1
            minor = 0
            patch = 0
        elif bump_type == "minor":
            minor += 1
            patch = 0
        else:  # patch
            patch += 1

        return f"{major}.{minor}.{patch}"

def main():
    parser = argparse.ArgumentParser(description="Gestionnaire de releases FTP_CALCULATOR")
    parser.add_argument("action", choices=["version", "prepare", "release", "changelog"],
                        help="Action à effectuer")
    parser.add_argument("--bump", choices=["major", "minor", "patch"], default="patch",
                        help="Type d'incrément de version (défaut: patch)")
    parser.add_argument("--version", help="Version spécifique (au lieu de l'incrément automatique)")
    parser.add_argument("--dry-run", action="store_true",
                        help="Mode simulation (ne fait pas les modifications)")

    args = parser.parse_args()

    manager = ReleaseManager()

    try:
        current_version = manager.get_current_version()
        print(f"📋 Version actuelle: {current_version}")

        if args.action == "version":
            print(f"🏷️  Version actuelle: {current_version}")

        elif args.action == "prepare":
            # Valide juste l'état du projet
            manager.validate_release_readiness()

        elif args.action == "changelog":
            new_version = args.version or manager.bump_version(current_version, args.bump)
            manager.generate_changelog(new_version)

        elif args.action == "release":
            if args.version:
                new_version = args.version
            else:
                new_version = manager.bump_version(current_version, args.bump)

            print(f"🎯 Nouvelle version: {new_version}")

            if not manager.create_release(new_version, args.bump, args.dry_run):
                sys.exit(1)

    except Exception as e:
        print(f"💥 Erreur: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()