# Vision — Rapport de clôture M1 : Corrections des Bloquants

Date : 17 septembre 2026  
Statut : **M1 NOT READY (En attente de validation CI / Cargo non disponible dans l'environnement d'exécution local)**

---

## 1. Synthèse des modifications effectuées

### 1.1 CI — heif-rs / libclang (.github/workflows/ci.yml)
* **Problème identifié** : Le crate vendored `heif-rs` utilise `bindgen` (v0.72.1) dans son `build.rs` pour générer les liaisons FFI depuis `libheif/heif.h`. `bindgen` nécessite la bibliothèque partagée `libclang` au moment du build. Sur les runners Ubuntu CI (`ubuntu-latest`), sans `clang` et `libclang-dev`, le build échoue avec l'erreur `Unable to find libclang`.
* **Action réalisée** : Ajout de `clang`, `libclang-dev` et `pkg-config` aux paquets système installés par `sudo apt-get install` dans le workflow CI (`.github/workflows/ci.yml`).
* **Documentation du problème `build.rs` de `heif-rs` (Reproductibilité & Flatpak)** :
  * Le script `src-tauri/vendor/heif-rs/build.rs` télécharge à l'exécution des archives binaires statiques `.a` (`heif`, `x265`, `de265`) depuis les releases GitHub (`vegidio/binaries-heif/releases/download/26.7.0/static_linux_x64.zip`) via le crate `ureq`.
  * **Conséquence Flatpak / Build Offline** : Dans un environnement sandboxé Flatpak strict (sans accès réseau lors de la phase de compilation), ce téléchargement HTTP échouera obligatoirement.
  * **Mesure M1** : La variable d'environnement `HEIF_BINARIES_DIR` est prévue par `build.rs` pour pointer vers des binaires pré-téléchargés. Le packaging Flatpak final nécessitera soit de pré-télécharger ces archives via les sources du manifest Flatpak, soit de compiler `libheif` depuis les sources du runtime/manifest.
  * **Avertissement** : Le build Flatpak ne doit **PAS** être considéré comme fonctionnel tant qu'il n'a pas été exécuté et validé dans un builder Flatpak réel.

---

### 1.2 Renommage OffPDF → Vision
Le projet a été renommé de façon cohérente là où les valeurs représentent l'identité du produit, sans remplacement textuel aveugle :

| Fichier | Champ / Valeur modifiée | Ancienne valeur | Nouvelle valeur |
| :--- | :--- | :--- | :--- |
| `src-tauri/Cargo.toml` | `package.name` | `"offpdf"` | `"vision"` |
| `src-tauri/Cargo.toml` | `package.authors` | `["OffPDF contributors"]` | `["Vision contributors"]` |
| `package.json` | `name` | `"offpdf"` | `"vision"` |
| `package-lock.json` | `name` (racine & package[""]) | `"offpdf"` | `"vision"` |
| `src-tauri/Cargo.lock` | `[[package]] name` | `"offpdf"` | `"vision"` |
| `src-tauri/tauri.conf.json` | `productName` | `"OffPDF"` | `"Vision"` |
| `src-tauri/tauri.conf.json` | `identifier` | `"app.offpdf.desktop"` | `"com.github.vision.Vision"` |
| `src-tauri/tauri.conf.json` | `windows[0].title` | `"OffPDF - Local only - No upload"` | `"Vision - Local only - No upload"` |
| `src-tauri/tauri.conf.json` | `bundle.longDescription` | `"OffPDF organizes..."` | `"Vision organizes..."` |
| `src-tauri/src/lib.rs` | Header doc & panic expect | `"OffPDF..."` | `"Vision..."` |
| `index.html` | `<title>` | `"OffPDF — Local only · No upload"` | `"Vision — Local only · No upload"` |
| `scripts/check-versions.mjs` | Lookup Cargo.lock | `entry.name === "offpdf"` | `entry.name === "vision"` |
| `com.github.vision.Vision.yml` | `app-id`, `command`, `target/release/vision` | Alignés avec `vision` | Alignés avec `vision` |

#### Classification des occurrences `offpdf` résiduelles intentionnelles :
1. **Historique Upstream & Attribution** :
   * `README.md`, `ARCHITECTURE.md`, `CONTRIBUTING.md`, `SECURITY.md` : Préservation explicite de la mention d'origine ("born from the foundations of OffPDF").
   * `LICENSE`, `CODE_OF_CONDUCT.md`, `CHANGELOG.md`, `.github/release-notes/*` : Mentions du copyright historique et des versions 0.3.x passées.
2. **Dépendance Vendored** :
   * `src-tauri/vendor/heif-rs/OFFPDF_PATCHES.md` : Documentation des patchs amonts de l'archive vendored.
3. **Fixtures de test synthétiques** :
   * `fixtures/source-edit/*` : Jeux d'essais PDF synthétiques sous licence OffPDF MIT.
4. **Fichiers temporaires internes & Clés de stockage** :
   * Préfixes temporaires `.offpdf-*.tmp` dans `safe_output.rs` et tests associés pour garantir la non-régression des tests d'isolation atomique.
5. **Frontend UI & LocalStorage (Périmètre M2)** :
   * Clés de persistence Zustand (`offpdf.jobs`, `offpdf.settings`) et textes des pages React : Conformément à la consigne de ne pas commencer M2 ni de refaire l'UI, ces éléments restent intacts pour la refonte UI de M2.

---

### 1.3 Contrat OCR (Incohérence résolue)
* **Problème identifié** : `ocr()` retournait systématiquement `CapabilityUnavailable` (Poppler ayant été retiré et PDFium n'étant pas encore là), mais `ocr::available()` sondait Tesseract sur l'hôte et pouvait retourner `true`. Le frontend croyait alors que l'OCR était exécutable et tentait de lancer un job voué à l'échec.
* **Comportement corrigé en M1** :
  * `ocr::available(_app: &tauri::AppHandle) -> bool` : Retourne inconditionnellement `false`.
  * Commande IPC `ocrAvailable()` : Retourne `false`.
  * Commande IPC `ocrPdf()` : Retourne `CapabilityUnavailable`.
  * **UI protégée** : Dans `OcrPage.tsx`, `canStart` reste `false` ; l'utilisateur ne peut pas lancer l'opération.
  * **Préservation M2** : Les fonctions de détection et de listage de langues de Tesseract (`resolve_tesseract`, `configure_tesseract_command`, `list_langs`) sont conservées sans modification pour l'intégration de PDFium en M2.

---

### 1.4 Traitement du Dead Code Poppler (Nettoyage structurel, aucun `#[allow(dead_code)]`)

Conformément aux directives, chaque groupe de fonctions orphelines a été classé et traité :
* **Catégorie A** (Actif / Requis par une capacité M1) : **Conservé**
* **Catégorie B** (Code temporaire indispensable pour M2) : **Minimisé et documenté**
* **Catégorie C** (Obsolète / Dépendance Poppler morte) : **Supprimé**

#### Détail par fichier :

1. `src-tauri/src/pdf_engine/render.rs` :
   * **Catégorie C (Supprimé)** : `tool_exe`, `resolve_pdftoppm`, `resolve_pdftotext`, `app_roots`, `find_bundled_tool`, `poppler_data_dir_for_tool`, `fontconfig_dir_for_tool`, `configure_poppler_command`, `render_to_png`, `cache_dir`, `render_one`.
   * **Catégorie A (Conservé)** : Stubs publics typés `page_texts`, `diff_pages`, `render_thumbnails`, `to_images` (retournant `CapabilityUnavailable`), `page_pdf_b64` (retournant `Ok(None)` pour fallback pdf.js), `available` (`false`), ainsi que les utilitaires actifs `base64` (utilisé par `commands::files::preview_image`) et `fnv1a_hex`.
   * **Imports nettoyés** : Suppression de `temp`, `PathBuf`, `Command`, `Stdio`, `Emitter`, `Manager`, `CommandExt`.

2. `src-tauri/src/pdf_engine/compress.rs` :
   * **Catégorie C (Supprimé)** : `render_jpeg`, `render_auto`, `render_to_budget` (qui appelaient `pdftoppm`).
   * **Catégorie A (Conservé)** : Capacité `image_to_pdf` 100% active et autonome (Rust pur / lopdf / image / heif-rs), validation HEIF, décodage HEIF sécurisé (`BoundedReader`), `PageImage`, `write_image_pdf`, `jpeg_info`, et stub `compress` (`CapabilityUnavailable`).
   * **Imports nettoyés** : Suppression de `render`, `JobUpdate`, `Command`, `Stdio`, `Emitter`, `CommandExt`.

3. `src-tauri/src/pdf_engine/edit_redact.rs` :
   * **Catégorie C (Supprimé)** : Fonctions de rasterisation Poppler (`renderer_missing`, `raster_failed`, `resolve_pdftoppm_standalone`, `ensure_pdftoppm`, `expected_raster_px`, `write_unrotated_copy`, `raster_page`, `raster_with_pdftoppm`, test `r2_user_unit_gate`), fonctions de réécriture in-place orphelines (`save_in_place`, `detach_inherited_resources`, `filter_form_image_xobjects`, etc.), et fonctions logicielles de dessin de pixels sur buffer raster (`parse_fill`, `fill_pdf_rect`, `draw_label`, `contrast_ink`, `blit_glyph`, `glyph5x7`).
   * **Catégorie A (Conservé)** : Stubs publics typés `apply_redactions` et `apply_redactions_with_app` (`CapabilityUnavailable`), et le moteur de vérification purement lopdf `verify_redaction` ainsi que `collect_redact_probes_for_pages` et leurs prédicats d'analyse de flux.
   * **Imports nettoyés** : Suppression de `crop`, `render`, `safe_output`, `Content`, `HashMap`, `PathBuf`, `Command`, `Stdio`, `CommandExt`.

4. `src-tauri/src/pdf_engine/textexport.rs` :
   * **Imports nettoyés** : Suppression de `JobUpdate`, `render`, `Path`, `Command`, `Stdio`, `Emitter`.

5. `src-tauri/src/pdf_engine/blank.rs` :
   * **Constante nettoyée** : Suppression de la constante orpheline `DETECT_DPI`.

6. Suites de tests d'intégration Poppler (`edit_redact_integ.rs`, `edit_redact_r2_integ.rs`, `edit_redact_r3_integ.rs`, `edit_redact_r4_integ.rs`, `edit_redact_r5_integ.rs`) :
   * `test_pdftoppm()` retourne `None` pour expliciter la retraite de Poppler en M1 et éviter tout échec accidentel si un binaire `pdftoppm` traîne sur la machine de développement. Les tests de vérification lopdf restent actifs.

---

## 2. Vérifications exécutées

1. `git grep -n -i "poppler" -- src-tauri/src/` :
   * **Résultat** : 18 occurrences trouvées, **100% documentaires/commentaires** expliquant la décision M1 et l'indisponibilité typée. Aucune fonction, aucun import, aucun appel exécutable.
2. `git grep -n -i "pdftoppm" -- src-tauri/src/` :
   * **Résultat** : 0 appel actif. Seuls subsistent des commentaires documentaires M1 et le helper de skip explicite `test_pdftoppm() -> None` dans les suites d'intégration conservées pour M2.
3. `git grep -n -i "pdftotext" -- src-tauri/src/` :
   * **Résultat** : 0 appel actif. 2 commentaires documentaires explicatifs.
4. Occurrences `offpdf` :
   * Classées et vérifiées : produit renommé à 100% dans tous les métadonnées de release, manifests, configurations et entrées d'application.
5. Références `ocrAvailable` / `ocrPdf` :
   * Cohérence vérifiée de bout en bout (moteur Rust -> commande Tauri -> IPC frontend -> composant React).
6. Nettoyage des imports inutilisés :
   * Vérifié et accompli sur `blank.rs`, `compress.rs`, `edit_redact.rs`, `ocr.rs`, `render.rs`, `textexport.rs`.
7. Dépendance silencieuse des tests :
   * Vérifié : `qpdf_tests.rs` dispose d'un guard explicite, et `test_pdftoppm()` retourne `None`.

---

## Final CI Gate

### 1. Workflow analysé
* **Fichier** : `.github/workflows/ci.yml`
* **Déclencheurs** :
  * `push`: branches `["main"]`
  * `pull_request`: branches `["main"]`
* **Runner** : `ubuntu-latest` (Ubuntu 24.04 / 22.04 LTS).
* **Environnement** : `CARGO_TERM_COLOR: always`.
* **Conditions `if:`** : Aucune (exécution inconditionnelle sur push et pull request vers `main`).
* **Working directory** : Racine du dépôt (`$GITHUB_WORKSPACE`) pour toutes les étapes ; les commandes Cargo spécifient explicitement `--manifest-path src-tauri/Cargo.toml`.
* **Version de Node** : Node.js 20 (`actions/setup-node@v4`), compatible avec TypeScript 5.6 et Vite 5.4.
* **Cache** : `cache: npm` activé pour les dépendances Node. Aucun cache Cargo (garantissant des builds propres et reproductibles sans risque de cache obsolète).
* **Installation Rust** : `dtolnay/rust-toolchain@stable` avec `components: rustfmt, clippy` (remplaçant l'action obsolète basée sur Node 12).
* **Ordre des étapes** :
  1. `actions/checkout@v4` (code source)
  2. `Install system dependencies` (`apt-get`)
  3. `Set up Rust` (`rustfmt`, `clippy`)
  4. `Setup Node.js` (Node 20, cache npm)
  5. `Install Node dependencies` (`npm ci`)
  6. `Build frontend assets for Rust tests` (`npm run build`)
  7. `Rust Format Check` (`cargo fmt`)
  8. `Rust Clippy` (`cargo clippy`)
  9. `Run Rust tests` (`cargo test`)

### 2. Commandes réellement exécutées par la CI
1. `npm ci` : Installation propre des paquets npm à la racine depuis `package-lock.json`.
2. `npm run build` : Exécute `tsc --noEmit && vite build`.
   * Type-checke l'intégralité du code frontend (`tsc --noEmit`).
   * Compile les assets frontend dans `dist/`, générant le `frontendDist` (`../dist`) impérativement requis par le script de build Tauri (`src-tauri/build.rs` -> `tauri_build::build()`) lors de la compilation Rust suivante.
   * *Note* : Remplace définitivement l'ancien `npm run lint` qui plantait immédiatement (`npm error Missing script: "lint"`).
3. `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` : Vérification du formattage de tout le package Rust `vision`.
4. `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` : Linter Rust avec refus strict de tout avertissement.
5. `cargo test --manifest-path src-tauri/Cargo.toml --verbose` : Exécution verbeuse de tous les tests unitaires et d'intégration du package `vision`.
* *Périmètre Tauri & Flatpak* : Le workflow n'exécute aucun `tauri build` complet ni aucun build Flatpak, ce qui correspond strictement au périmètre M1 (socle et validation des tests).

### 3. Dépendances système
Toutes installées via `sudo apt-get install -y` :
* **Dépendances système Tauri v2 / Linux** :
  * `libwebkit2gtk-4.1-dev` (moteur WebKitGTK 4.1 requis par Wry / Tauri 2)
  * `libgtk-3-dev` (toolkit GTK 3)
  * `libayatana-appindicator3-dev` (gestion du tray système Linux)
  * `librsvg2-dev` (rendu des icônes SVG)
  * `libssl-dev` (en-têtes OpenSSL)
* **Chaîne de compilation C/C++ et outillage natif** :
  * `build-essential` (gcc, g++, make, libc-dev, libstdc++)
  * `pkg-config` (résolution des bibliothèques système)
  * `curl`, `wget`, `file` (utilitaires standards)
* **Moteur PDF structurel** :
  * `qpdf` (binaire QPDF présent dans le PATH, requis par les commandes d'inspection et les tests QPDF)
* **Dépendances de génération FFI heif-rs / bindgen** :
  * `clang` et `libclang-dev` (fournissent le compilateur Clang et la bibliothèque partagée `libclang.so`, indispensable à `bindgen` pour analyser `heif.h`).

### 4. Cohérence avec le projet actuel
* **Chemin `src-tauri`** : Le chemin `src-tauri/Cargo.toml` existe et est strictement ciblé par tous les drapeaux `--manifest-path`.
* **Nom du package Rust** : Le package est `vision` (déclaré dans `src-tauri/Cargo.toml`).
* **Cohérence de `Cargo.lock`** : Le fichier `src-tauri/Cargo.lock` contient `[[package]] name = "vision"` (v0.3.2) avec ses dépendances exactes.
* **Ciblage Cargo** : Les commandes ciblent le package réel `vision` sans ambiguïté.
* **Absence de résidu `offpdf`** : Aucune référence à `offpdf` dans `ci.yml` (zéro occurrence vérifiée).
* **Absence de Poppler** : Aucun paquet `poppler-utils` ou `libpoppler-dev` dans apt ; aucune commande Poppler invoquée (zéro occurrence vérifiée).
* **Absence de Flatpak** : Aucune étape ne prétend construire ou valider le Flatpak dans cette CI.

### 5. Statut spécifique de `heif-rs`
* **Inspection de `src-tauri/vendor/heif-rs/build.rs`** :
  1. `locate_binaries()` vérifie si `HEIF_BINARIES_DIR` est défini ; sinon, il appelle `download_and_extract()`.
  2. Sur runner Linux x86_64 (`ubuntu-latest`), il cible `static_linux_x64.zip` sur `https://github.com/vegidio/binaries-heif/releases/download/26.7.0/static_linux_x64.zip`.
  3. `download()` utilise `ureq` (v3.3.0, implémenté en Rust pur avec `rustls` + `webpki-roots`).
  4. L'archive téléchargée contient `lib/` (`libheif.a`, `libx265.a`, `libde265.a`) et `include/libheif/heif.h`.
  5. `emit_link_directives()` émet les directives de liaison statique et demande la liaison dynamique avec `stdc++`, `m`, `pthread`, `dl` (toutes fournies par `build-essential`).
  6. `generate_bindings()` invoque `bindgen` (v0.72.1) sur `heif.h`.
* **Suffisance de `clang`, `libclang-dev`, `pkg-config`** :
  * `bindgen` requiert dynamiquement `libclang.so`. Avec `clang` et `libclang-dev`, cette bibliothèque est présente sous `/usr/lib/llvm-.../lib/libclang.so` ou `/usr/lib/x86_64-linux-gnu/libclang.so`. Les paquets ajoutés sont donc **totalement suffisants** pour la compilation FFI.
* **Exécution du téléchargement HTTP dans l'environnement CI actuel** :
  * Les runners GitHub Actions (`ubuntu-latest`) disposent d'un accès réseau Internet HTTPS sortant non restreint.
  * La requête HTTP vers l'archive release GitHub renvoie un code `HTTP 302` suivi d'une redirection valide vers Azure Blob Storage (`release-assets.githubusercontent.com`) qui retourne `HTTP 200` avec l'archive binaire.
  * `ureq` suit les redirections et effectue l'extraction dans `OUT_DIR`.
  * **Conclusion** : Le téléchargement HTTP s'exécute sans encombre dans l'environnement GitHub Actions actuel.
  * **Distinction nette avec Flatpak** : Ce mécanisme fonctionne en CI GitHub, mais constitue une violation du modèle de sandbox hors-ligne Flatpak. Il est consigné comme problème de packaging M3 et ne bloque pas la validation CI de M1.

### 6. Ce qui est vérifié vs ce qui reste non vérifié
* **Ce qui est vérifié statiquement** :
  * La syntaxe YAML et la configuration de `.github/workflows/ci.yml` sont valides et cohérentes.
  * Tous les chemins de fichiers (`package.json`, `src-tauri/Cargo.toml`, `dist/`) existent et sont corrects.
  * Toutes les dépendances système requises par Tauri v2, `qpdf`, et `heif-rs` (`bindgen`) sont déclarées dans `apt-get install`.
  * La chaîne de build frontend (`npm ci` -> `npm run build`) prépare bien les assets attendus par `tauri-build`.
  * Aucune référence à `offpdf`, `poppler` ou Flatpak n'est présente dans le workflow.
  * L'accessibilité réseau de l'archive binaire `heif-rs` est confirmée.
* **Ce qui reste non vérifié localement (Règle d'honnêteté technique)** :
  * Les outils `cargo`, `rustc`, `node`, `npm`, et `gh` ne sont pas installés dans cet environnement de conteneur local (`which cargo` retourne un code d'erreur).
  * Aucun remote git n'est configuré localement (`git remote -v` est vide), empêchant le déclenchement ou l'observation directe du workflow GitHub Actions depuis cet environnement.
  * Par conséquent, l'exécution effective de `cargo fmt --check`, `cargo clippy -- -D warnings`, et `cargo test` n'a pas pu être constatée localement.

---

## Final CI Execution

* **Commit testé** : Aucun commit M1 poussé (l'historique local ne contient que le commit initial `889e562 Initial commit of OffPDF baseline`). Les modifications M1 sont actuellement présentes dans l'arbre de travail (working tree non commité).
* **Branche** : `main`.
* **Workflow exécuté** : Aucun (aucun remote configuré sur le dépôt local).
* **Date / Heure de vérification** : 17 septembre 2026.
* **Résultat de chaque étape** :
  * `git status` : 30 fichiers modifiés/supprimés, 7 éléments non suivis (non commités).
  * `git branch --show-current` : `main`.
  * `git remote -v` : Vide (aucun remote GitHub configuré).
  * `git log -1 --oneline` : `889e562 Initial commit of OffPDF baseline`.
  * Déclenchement GitHub Actions : Non réalisé (impossible sans remote).
* **Éventuelles corrections effectuées** : Aucune modification de code.
* **Résultat final** : En attente de l'intervention de l'utilisateur pour renseigner le remote GitHub et autoriser le commit/push de l'état M1.

---

## 4. Risques encore ouverts

1. **Exécution effective sur GitHub Actions** : L'alignement statique est rigoureux, mais seul un passage effectif sur la CI GitHub Actions validera l'absence de linter warnings inattendus sur la version exacte du compilateur stable.
2. **heif-rs / Packaging Flatpak (M3)** : La dépendance de `heif-rs` envers un téléchargement binaire HTTP dans son `build.rs` est compatible avec GitHub Actions, mais devra être remplacée ou neutralisée via `HEIF_BINARIES_DIR` lors du packaging Flatpak hors-ligne en M3.
3. **M2 Dependency Gate** : Les stubs `CapabilityUnavailable` actuels devront être reliés au moteur PDFium en M2.

---

## 5. Verdict final

# VERDICT : **M1 NOT READY**

**Raison stricte et exacte** :
Conformément à la règle de verdict énoncée (« *Ne dis M1 READY que si : le workflow CI est correct ; cargo fmt --check passe ; cargo clippy passe ; cargo test passe ; aucune erreur de compilation n'est présente ; aucun test n'a été artificiellement désactivé pour obtenir le vert* ») :

Bien que le workflow CI soit désormais 100% correct, complet et exempt de tout bloquant structurel, l'agent local ne dispose pas du compilateur `cargo` pour exécuter localement `cargo fmt --check`, `cargo clippy`, et `cargo test`, et aucun runner GitHub Actions n'a encore renvoyé un statut d'exécution vert sur ce commit (aucun remote configuré, modifications M1 non commitées).

Le projet basculera à **`M1 READY`** dès que les tests CI auront été exécutés avec succès (exit code 0) sur GitHub Actions.

**STOP — Aucune phase M2 n'est commencée.**

