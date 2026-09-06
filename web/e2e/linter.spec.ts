import { expect, test } from '@playwright/test'

test('lints a clean sentence and renders the parse tree', async ({ page }) => {
  await page.goto('/')

  await expect(page.getByRole('heading', { name: 'angloform' })).toBeVisible()

  // the default sentence is linted on load.
  await expect(page.getByText('parses uniquely')).toBeVisible()

  // parse tree chart renders nodes.
  // parse tree chart renders the tree with full phrase names.
  const svg = page.locator('svg').first()
  await expect(svg).toContainText('Statement')
  await expect(svg).toContainText('reads')
  await expect(svg).toContainText('a determiner')
 await expect(page.getByText('Open Dependencies')).toBeVisible()
})

test('shows a rejection for a banned pronoun', async ({ page }) => {
  await page.goto('/')
  await page.getByLabel('Examples').getByText('banned pronoun').click()
  await expect(page.getByText('"it" is banned in angloform')).toBeVisible()
})

test('types a sentence and lints it on Ctrl+Enter', async ({ page }) => {
  await page.goto('/')
  const textarea = page.getByPlaceholder('type a sentence of angloform…')
  await textarea.fill('the queue is empty')
  await textarea.press('Control+Enter')
  await expect(page.getByText('parses uniquely')).toBeVisible()
  await expect(page.locator('span').filter({ hasText: /^empty$/ })).toBeVisible()
})