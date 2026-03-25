import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import * as Localization from 'expo-localization';

// Translations
import pl from './locales/pl.json';
import en from './locales/en.json';

const resources = {
  pl: { translation: pl },
  en: { translation: en },
};

// Get device locale
const deviceLocale = Localization.locale.split('-')[0];
const defaultLocale = ['pl', 'en'].includes(deviceLocale) ? deviceLocale : 'pl';

i18n
  .use(initReactI18next)
  .init({
    resources,
    lng: defaultLocale,
    fallbackLng: 'en',
    interpolation: {
      escapeValue: false,
    },
    react: {
      useSuspense: false,
    },
  });

export default i18n;
