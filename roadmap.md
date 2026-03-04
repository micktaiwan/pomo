# Roadmap

## V0 — Timer basique

CLI qui démarre un countdown dans le terminal avec affichage ASCII gros chiffres.

```bash
pomo 25m
pomo 90s
pomo 1h30m
```

- [x] Parse des durées (secondes, minutes, heures)
- [x] Affichage countdown en gros chiffres ASCII
- [x] Rafraîchissement en temps réel dans le terminal
- [x] Notification macOS native à la fin (via osascript)
- [x] Son à la fin
- [x] Mode chronomètre (pomo sans argument)
- [x] Support des jours (`pomo 2j`)
- [x] Affichage heure de début
- [x] terminal-notifier (fallback osascript)
- [x] Heure cible (`pomo 14:30`)
- [x] Option `--title` pour afficher un texte au-dessus du timer
- [x] Option `-s` pour la taille d'affichage (1=texte, 2=compact, 3=large)

## V1 — Essentiels

- [x] Pause/resume (espace ou `p`)
- [ ] Cycles work/break (`pomo 25m/5m` ou `pomo 4x25m/5m/15m`)
- [ ] Couleurs (rouge quand le temps est bientôt écoulé, vert pendant les pauses)

## V2 — Suivi

- [ ] Barre de progression sous les chiffres ASCII
- [ ] Historique des sessions (`~/.pomo/history` : timestamp, durée, complété/annulé)
- [ ] Intégration tmux (`pomo status` pour `#(pomo status)`)

## V3 — Extras

- [ ] Stats (`pomo stats` : résumé jour/semaine depuis l'historique)
- [ ] Fichier de config (`~/.pomo/config` pour les durées par défaut)
- [ ] Hooks (exécuter un script au début/fin de session)
- [ ] Label/tag de session (`pomo 25m "code review"`)
