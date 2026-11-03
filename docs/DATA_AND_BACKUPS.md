# Données, profils et sauvegardes

## SQLite

SQLite contient les données relationnelles et métadonnées :
- profils et paramètres ;
- progression, XP, compétences, trophées ;
- historique d'activités et objectifs ;
- cours et index de recherche si nécessaire ;
- notes, favoris, exercices à reprendre ;
- dépôts Git et état de synchronisation ;
- conversations IA et messages ;
- statistiques locales ;
- métadonnées de laboratoires, snapshots et sauvegardes ;
- notifications et préférences.

Les fichiers volumineux restent dans le stockage géré : archives, disques VM, volumes, enregistrements de sessions et assets.

## Profils

- Profils multiples sur la même installation.
- Aucun compte distant.
- Création, sélection, renommage et suppression avec confirmation.
- Chaque profil conserve sa progression, ses notes et préférences.
- Les laboratoires sont associés au profil et au scénario.

## Migrations

- Migrations versionnées.
- Tester les migrations depuis chaque version supportée.
- Transactions et stratégie de récupération.
- Sauvegarde avant migration risquée.
- Ne jamais supprimer silencieusement des données incompatibles.

## Racine de stockage

Répertoire racine configurable, structuré par profil/labo/machine/snapshot. Les chemins internes gérés par Docker/Podman/hyperviseurs ne doivent pas être déplacés arbitrairement.

## Archives de sauvegarde

- Format compressé portable.
- Compression Zstandard avec niveau équilibré.
- Métadonnées de version et manifeste.
- Vérification d'intégrité.
- Restauration complète ou sélective.
- Destinations : local, disque USB, NAS/SMB/NFS.
- Pas de chiffrement intégré ; expliquer que la confidentialité dépend des permissions et du chiffrement du stockage hôte.

## Sauvegarde

Inclure :
- profil et paramètres ;
- SQLite ;
- scénarios personnels et projets ;
- conversations IA et notes ;
- volumes de laboratoires ;
- disques VM ;
- snapshots et métadonnées ;
- enregistrements de sessions.

La sauvegarde multi-machine doit être cohérente : coordonner l'arrêt ou la mise en pause lorsque nécessaire et expliquer les limites.

## Fréquence et rétention

- Sauvegardes automatiques configurables et manuelles.
- Automatisation désactivable.
- Historique et état visibles.
- Rétention globale et par labo configurable.
- Sauvegardes épinglées protégées.
- Avertir avant nettoyage automatique.

## Restauration sûre

1. Vérifier intégrité.
2. Vérifier compatibilité de version et capacités.
3. Afficher taille et contenu.
4. Prévisualiser les données qui seront remplacées.
5. Demander confirmation.
6. Créer une sauvegarde de l'état actuel.
7. Vérifier que la sauvegarde de sécurité est utilisable.
8. Restaurer.
9. Proposer rollback si échec.
10. Journaliser localement.

Ne jamais remplacer des données sans avertissement clair.
