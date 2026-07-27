module.exports = {
  theme: {
    fontFamily: {
      sans: [
        'Inter',
        'Cantarell',
        'Roboto',
        'system-ui',
        '-apple-system',
        'BlinkMacSystemFont',
        '"Segoe UI"',
        '"Helvetica Neue"',
        'Arial',
        '"Noto Sans"',
        'sans-serif',
        '"Apple Color Emoji"',
        '"Segoe UI Emoji"',
        '"Segoe UI Symbol"',
        '"Noto Color Emoji"',
      ],
    },
    extend: {
      inset: {
        '1/8': '0.125em',
      },
      boxShadow: {
        even: '0 2px 12px 0 #0000001a',
        error: '0 0 0 3px #F56C6CA0',
      },
      padding: {
        '2.5': '0.625em',
        '3/2': '0.375em',
      },
      colors: {
        surface: 'var(--surface)',
        raised: 'var(--raised)',
        edge: 'var(--edge)',
        fg: 'var(--fg)',
        muted: 'var(--muted)',
        accent: {
          DEFAULT: 'var(--accent)',
          fg: 'var(--accent-fg)',
        },
        selected: 'var(--selected)',
        ok: 'var(--ok)',
        warn: 'var(--warn)',
        danger: 'var(--danger)',
        success: 'var(--success)',
      },
      height: {
        min: 'min-content',
      },
    },
  },
  plugins: [require('@tailwindcss/forms')],
};
