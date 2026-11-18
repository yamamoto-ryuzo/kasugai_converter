// Kasuga Converter ドキュメント共通ナビゲーション
// 使い方: <script src="nav.js" defer></script>
// - 左サイドバーにサイト内リンクを生成
// - 現在ページを自動ハイライト
// - JS が無効でも本文は通常表示される（サイドバーだけ出ない）
// - index.html が存在すれば .html（GitHub Pages）、無ければ実体の .md へリンクする
//   （file:// 直開きやローカル HTTP サーバーでは .md しか存在しないため）
(async function () {
  const VERSION = 'v0.8.3';

  async function detectExt() {
    if (location.protocol === 'file:') return '.md';
    try {
      const r = await fetch('index.html', { method: 'HEAD' });
      return r.ok ? '.html' : '.md';
    } catch (_) {
      return '.md';
    }
  }
  const EXT = await detectExt();

  // href は拡張子なしで書き、生成時に EXT を付ける（3dtiles は常に .html の実体）
  const GROUPS = [
    {
      section: '概要',
      sectionHref: 'index',
      items: [
        { href: 'index', text: 'トップ' },
      ]
    },
    {
      section: '使い方',
      items: [
        { href: 'setup', text: 'セットアップ' },
        { href: 'usage', text: '使い方' },
      ]
    },
    {
      section: '3D Tiles',
      items: [
        { href: '3dtiles.html', text: '3D Tiles 変換ガイド' },
      ]
    },
    {
      section: 'その他',
      items: [
        { href: 'license', text: 'ライセンス' },
      ]
    },
  ];

  const urlFor = (href) => href.includes('.') ? href : href + EXT;

  const current = location.pathname.replace(/\/$/, 'index.html').split('/').pop() || 'index.html';

  const nav = document.createElement('nav');
  nav.id = 'site-nav';

  let html = '<div class="nav-title"><a href="' + urlFor('index') + '">Kasuga Converter Docs</a>'
    + '<span class="nav-version">' + VERSION + '</span></div>';

  for (const g of GROUPS) {
    html += '<div class="nav-section">'
      + (g.sectionHref ? '<a href="' + urlFor(g.sectionHref) + '">' + g.section + '</a>' : g.section)
      + '</div>';
    if (g.items && g.items.length) {
      html += '<ul>';
      for (const it of g.items) {
        const active = urlFor(it.href) === current ? ' class="active"' : '';
        html += '<li><a href="' + urlFor(it.href) + '"' + active + '>' + it.text + '</a></li>';
      }
      html += '</ul>';
    }
  }
  nav.innerHTML = html;
  document.body.appendChild(nav);
  document.body.classList.add('has-nav');

  const toggle = document.createElement('button');
  toggle.className = 'nav-toggle';
  toggle.textContent = '☰';
  toggle.setAttribute('aria-label', 'ナビゲーション');
  toggle.addEventListener('click', () => document.body.classList.toggle('nav-open'));
  document.body.appendChild(toggle);

  // ページ内の「.html 固定リンク」もローカル環境では .md へ向け直す（相対リンクのみ）
  if (EXT === '.md') {
    document.querySelectorAll('a[href$=".html"]').forEach(a => {
      const href = a.getAttribute('href');
      if (!href.includes('://') && !href.startsWith('//') && href.split('/').pop() !== '3dtiles.html') {
        a.href = href.replace(/\.html$/, '.md');
      }
    });
  }
})();
