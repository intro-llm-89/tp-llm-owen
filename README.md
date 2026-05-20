# tp-llm-owen

## I - Installation

*L'installation ne parlera que de LM Studio, si vous utilisez Ollama ou un autre outil, vous devez vous adapter en conséquence.*

Donc vous aurez besoin de LM Studio pour la suite.

Je vous recommande fortement d'utiliser le LLM `gemma-4-e4b` de Google.

Pour la suite je vous recommande de faire la commande:
```powershell
lms server start
```

C'est tout pour l'installation coté LM Studio et des endpoints, pour ce qui est de ce repo vous aurez besoin des dépendances pour faire tourner du code Rust.
<!--
TODO: Ajoutez une release pour juste installer un exe
-->

Ensuite il vous suffit de cloner ou bien de fork le repo puis d'aller à la racine et de faire la commande:
```powershell
cargo run --release
```

Et vous avez le code de lancer, si vous avez des erreurs n'hésitez pas à faire une issue

## II - Utilisation

L'utilisation à été simplifiée via la crate [inquire](https://crates.io/crates/inquire) donc même sans lire cette section vous devriez vous en sortir.

Sinon l'outil vous permets de prompt une IA via un fichier ou votre terminal, puis d'envoyer une image ou un PDF à votre IA afin de l'analyser.

Vous pouvez ensuite choisir d'afficher oui ou non la réponse.

## III - Autre

Actuellement des améliorations sont en cours comme:
- Pick un ou des fichiers dont vous pouvez envoyer le contenu (png, jpeg, pdf) plutôt que de mettre le path à la main
- Enlever le markdown des réponses du LLM car non pris en compte dans l'affichage
