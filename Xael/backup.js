// Xael Automated Backup System
import { writeFileSync, mkdirSync, readdirSync, unlinkSync } from 'node:fs';
import { join } from 'node:path';

export function initBackups(dataDir) {
  const dir = join(dataDir, 'backups');
  try { mkdirSync(dir, { recursive: true }); } catch {}
  return dir;
}

export function runBackup(backupDir, stores) {
  try {
    const ts = new Date().toISOString().replace(/[:.]/g, '-');
    for (const [name, data] of stores) {
      writeFileSync(join(backupDir, name + '-' + ts + '.json'), JSON.stringify(data));
    }
    for (const prefix of stores.map(s => s[0])) {
      const files = readdirSync(backupDir).filter(f => f.startsWith(prefix + '-')).sort();
      while (files.length > 50) {
        try { unlinkSync(join(backupDir, files.shift())); } catch {}
      }
    }
  } catch (e) { console.error('[backup]', e.message); }
}
