import assert from 'node:assert/strict'
import { mkdtemp, readFile, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import path from 'node:path'
import test from 'node:test'

import { syncReleaseVersion } from '../scripts/sync-release-version.mjs'

async function writeFixture(root) {
  await writeFile(
    path.join(root, 'package.json'),
    JSON.stringify({ name: 'gigacoder-config-assistant', version: '0.1.1' }, null, 2) + '\n',
  )
  await writeFile(
    path.join(root, 'package-lock.json'),
    JSON.stringify(
      {
        name: 'gigacoder-config-assistant',
        version: '0.1.1',
        lockfileVersion: 3,
        packages: {
          '': { name: 'gigacoder-config-assistant', version: '0.1.1' },
        },
      },
      null,
      2,
    ) + '\n',
  )

  const tauriDir = path.join(root, 'src-tauri')
  await import('node:fs/promises').then(({ mkdir }) => mkdir(tauriDir, { recursive: true }))
  await writeFile(
    path.join(tauriDir, 'tauri.conf.json'),
    JSON.stringify({ productName: 'GigaCoderAssist', version: '0.1.1' }, null, 2) + '\n',
  )
  await writeFile(
    path.join(tauriDir, 'Cargo.toml'),
    '[package]\nname = "gigacoder-config-assistant"\nversion = "0.1.1"\nedition = "2021"\n',
  )
  await writeFile(
    path.join(tauriDir, 'Cargo.lock'),
    'version = 4\n\n[[package]]\nname = "gigacoder-config-assistant"\nversion = "0.1.1"\n',
  )
}

test('syncs package metadata to the release tag version', async () => {
  const root = await mkdtemp(path.join(tmpdir(), 'release-version-'))
  await writeFixture(root)

  await syncReleaseVersion({ root, tag: 'v0.1.0' })

  assert.equal(JSON.parse(await readFile(path.join(root, 'package.json'), 'utf8')).version, '0.1.0')
  assert.equal(
    JSON.parse(await readFile(path.join(root, 'package-lock.json'), 'utf8')).packages[''].version,
    '0.1.0',
  )
  assert.equal(
    JSON.parse(await readFile(path.join(root, 'src-tauri', 'tauri.conf.json'), 'utf8')).version,
    '0.1.0',
  )
  assert.match(await readFile(path.join(root, 'src-tauri', 'Cargo.toml'), 'utf8'), /version = "0\.1\.0"/)
  assert.match(await readFile(path.join(root, 'src-tauri', 'Cargo.lock'), 'utf8'), /version = "0\.1\.0"/)
})

test('succeeds when cargo metadata already matches the release tag', async () => {
  const root = await mkdtemp(path.join(tmpdir(), 'release-version-'))
  await writeFixture(root)

  await syncReleaseVersion({ root, tag: 'v0.1.0' })
  await syncReleaseVersion({ root, tag: 'v0.1.0' })

  assert.match(await readFile(path.join(root, 'src-tauri', 'Cargo.toml'), 'utf8'), /version = "0\.1\.0"/)
  assert.match(await readFile(path.join(root, 'src-tauri', 'Cargo.lock'), 'utf8'), /version = "0\.1\.0"/)
})

test('syncs cargo lock files checked out with windows newlines', async () => {
  const root = await mkdtemp(path.join(tmpdir(), 'release-version-'))
  await writeFixture(root)
  await writeFile(
    path.join(root, 'src-tauri', 'Cargo.lock'),
    'version = 4\r\n\r\n[[package]]\r\nname = "gigacoder-config-assistant"\r\nversion = "0.1.1"\r\n',
  )

  await syncReleaseVersion({ root, tag: 'v0.1.0' })

  assert.match(await readFile(path.join(root, 'src-tauri', 'Cargo.lock'), 'utf8'), /version = "0\.1\.0"/)
})

test('rejects tags that are not v-prefixed semantic versions', async () => {
  const root = await mkdtemp(path.join(tmpdir(), 'release-version-'))
  await writeFixture(root)

  await assert.rejects(() => syncReleaseVersion({ root, tag: 'release-0.1.0' }), /Expected tag/)
})
