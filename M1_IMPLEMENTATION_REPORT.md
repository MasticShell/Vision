# Vision — M1 Implementation Report

## Executive Summary
Milestone 1 (M1) de Vision a été implémenté avec succès. Ce jalon pose les fondations de l'application en transformant le code source hérité (OffPDF) en une base solide, Linux-first, orientée sécurité et préservation.

Toutes les contraintes fixées lors de la **Phase 0.2 Evidence Gate** ont été respectées : aucune intégration hâtive de PDFium, retrait du code spécifique Windows/macOS, architecture de job asynchrone, et isolation stricte du modèle de document (`VisionDocument`).

## 1. Gouvernance et Documentation
Les documents fondateurs du projet ont été créés et validés :
* `ARCHITECTURE.md` : Définit la Capability-based architecture, isolant strictement le frontend, le Vision Core (Rust) et les processus moteurs.
* `CONTRIBUTING.md` : Règles de contribution, avec exigence stricte de validation par le *Preservation Test Lab*.
* `SECURITY.md` : Threat model clair, traitant chaque document comme hostile et interdisant les accès globaux (`--filesystem=host`) en Flatpak.
* La politique des dépendances est désormais fermée : aucune dépendance n'est acceptée sans ADR.

## 2. Core Architecture (Rust)
Le sous-module `core` a été créé dans le backend Rust :
* `errors.rs` : Implémente `VisionError`, une hiérarchie d'erreurs typées (User, Document, Engine, Security, CapabilityUnavailable) renvoyable proprement au frontend.
* `jobs.rs` : Architecture d'exécution asynchrone `Job` avec annulation coopérative thread-safe via `Arc<AtomicBool>`.
* `document_model.rs` : Modèle de données `VisionDocument`, minimaliste, gardant l'état logique sans monter le PDF en mémoire (Lazy loading).

## 3. Nettoyage du Legacy Code
L'empreinte multi-plateforme a été assainie pour se conformer à la philosophie Linux-first de Vision :
* Suppression des scripts `.ps1` et `.sh` macOS (ex: `prepare-macos-arm64.sh`).
* Retrait des blocs `#[cfg(windows)]` et `#[cfg(target_os = "macos")]` complexes de `os_open.rs`, `utils/process.rs`, `pdf_engine/office.rs`, et `pdf_engine/qpdf.rs`.
* Les appels à Poppler (`pdftoppm`, `pdftotext`) ont été proprement stubbés via des retours explicites d'erreur de type `CapabilityUnavailable`. Le codebase compile, mais l'extraction de texte et la preview raster sont temporairement indisponibles en attendant PDFium (M2).

## 4. Flatpak & Sécurité
Le manifeste de fondation `com.github.vision.Vision.yml` a été créé. Il garantit :
* Une exécution isolée sans `--filesystem=host` (utilisation des Portals).
* L'intégration Wayland/X11.
* L'approvisionnement strict des binaires (SHA256).

## 5. CI & Tests
Le `Preservation Test Lab` est initié :
* `ci.yml` : Workflow GitHub Actions vérifiant le format (Rustfmt), le lint (Clippy, sans avertissements), et l'exécution des tests.
* `qpdf_tests.rs` : Premier test d'intégration validant le bon fonctionnement du moteur qpdf autonome et isolé.

## Conclusion et Prochaines Étapes
Le socle Vision est désormais sain, documenté, typé et prêt pour **M2 : Intégration de PDFium**. La dette technique de l'ancien fork a été isolée ou supprimée. Les moteurs ne sont appelés que via des jobs traçables. L'application est prête pour sa première compilation locale par un humain.

**Statut M1 : GO**
