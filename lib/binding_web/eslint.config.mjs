import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  eslint.configs.recommended,
  tseslint.configs.recommendedTypeChecked,
  tseslint.configs.strictTypeChecked,
  tseslint.configs.stylisticTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      'no-fallthrough': 'off',
      // TODO: re-enable when typescript-eslint supports ESLint v10 (https://github.com/typescript-eslint/typescript-eslint/issues/11952)
      '@typescript-eslint/consistent-generic-constructors': 'off',
      '@typescript-eslint/no-deprecated': 'off',
      '@typescript-eslint/no-non-null-assertion': 'off',
      '@typescript-eslint/no-unnecessary-condition': ['error', {
        allowConstantLoopConditions: true
      }],
      '@typescript-eslint/restrict-template-expressions': ['error', {
        allowNumber: true
      }],
    }
  },
);
