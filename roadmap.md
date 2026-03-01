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
