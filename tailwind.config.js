module.exports = {
  future: {
    purgeLayersByDefault: true,
    removeDeprecatedGapUtilities: true,
  },
  theme: {
    fontFamily: {
      sans: [
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
        '1.5': '0.375em',
      },
      colors: {
        ui: {
          background: 'var(--color-ui-background)',
          emph: 'var(--color-ui-emph)',
          sidebar: 'var(--color-ui-sidebar)',
          text: 'var(--color-ui-text)',
          'text-inv': 'var(--color-ui-text-inv)',
          primary: 'var(--color-ui-primary)',
          border: 'var(--color-ui-border)',
          blockquote: 'var(--color-ui-blockquote)',
        },
      },
    },
  },
  variants: {},
  plugins: [require('@tailwindcss/custom-forms')],
  purge: false,
};
