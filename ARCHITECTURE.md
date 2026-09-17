# Architecture de Vision

Vision est une application desktop Linux FOSS dédiée à la gestion avancée des PDF, distribuée principalement via Flatpak.
Elle est conçue avec une architecture **Capability-based** stricte.

## 1. Séparation des responsabilités (Vision Core vs Engines)

L'architecture repose sur un principe absolu :
> **Les moteurs (qpdf, PDFium, etc.) fournissent des capacités. Vision possède l'état logique du document et décide comment utiliser ces capacités.**

* **Frontend (React/TypeScript)** : Gère exclusivement la présentation et l'interaction utilisateur.
* **Vision Core (Rust)** : Détient l'état logique via le modèle `VisionDocument`, orchestre les transactions, gère le cache et les Jobs asynchrones.
* **Engines** : Processus externes isolés (CLI) ou bibliothèques embarquées, responsables uniquement de l'exécution de tâches très spécifiques (ex: rotation de page par qpdf, OCR par Tesseract).

## 2. Capability Matrix actuelle

Le choix d'un moteur se fait **par opération**, selon les critères de préservation, de performance et de sécurité :

* **Opérations structurelles (Merge, Split, Rotate, Delete)** : Déléguées à **qpdf (CLI)**. qpdf est notre garant de la "Préservation PDF" car il modifie l'arbre des objets sans altérer les flux originaux ni détruire les signatures s'il n'y a pas de raison de le faire.
* **OCR (Création de PDF textable)** : Délégué à **Tesseract (CLI)**.
* **Conversion Office vers PDF** : Déléguée temporairement à LibreOffice via `flatpak-spawn`.

*(Note : L'intégration de PDFium pour le rendu visuel et l'édition de contenu est prévue pour le Milestone M2).*

## 3. Sécurité et "Least Privilege"

Vision traite chaque fichier PDF comme potentiellement hostile (fuzzing, memory exhaustion, etc.).
* L'exécution des tâches lourdes se fait dans des processus enfants isolés (CLI) pour empêcher un crash C++ (ex: segfault) de tuer l'application principale.
* En Flatpak, Vision n'utilise **pas** l'accès `--filesystem=host`. L'accès aux fichiers est strictement contrôlé via les portails XDG ou confiné au dossier temporaire (`XDG_RUNTIME_DIR`).

## 4. Preservation Test Lab

Aucune modification de contenu n'est permise sans avoir été testée au préalable par notre infrastructure de tests automatisés. Toute opération (ex: Split) est validée sémantiquement et structurellement (`qpdf --check`) dans la CI pour garantir l'absence de régression silencieuse.
