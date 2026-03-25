/**
 * i18n configuration for RoadRunner Mobile App
 * 
 * Supports Polish and English languages with easy extensibility.
 * Uses i18next with react-i18next for React Native.
 */

import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import AsyncStorage from '@react-native-async-storage/async-storage';
import * as Localization from 'expo-localization';

// Import translations
import pl from './locales/pl.json';
import en from './locales/en.json';

const STORAGE_KEY = '@roadrunner_language';

// Available languages
export const LANGUAGES = {
  pl: {
    code: 'pl',
    name: 'Polski',
    flag: '🇵🇱',
  },
  en: {
    code: 'en',
    name: 'English',
    flag: '🇬🇧',
  },
};

export type LanguageCode = keyof typeof LANGUAGES;

// Translation resources
const resources = {
  pl: { translation: pl },
  en: { translation: en },
};

/**
 * Get stored language from AsyncStorage
 */
const getStoredLanguage = async (): Promise<string | null> => {
  try {
    return await AsyncStorage.getItem(STORAGE_KEY);
  } catch (error) {
    console.error('Error reading stored language:', error);
    return null;
  }
};

/**
 * Save language to AsyncStorage
 */
export const setStoredLanguage = async (language: string): Promise<void> => {
  try {
    await AsyncStorage.setItem(STORAGE_KEY, language);
  } catch (error) {
    console.error('Error storing language:', error);
  }
};

/**
 * Detect initial language based on stored preference or device locale
 */
const detectLanguage = async (): Promise<string> => {
  // First check stored preference
  const storedLang = await getStoredLanguage();
  if (storedLang && LANGUAGES[storedLang as LanguageCode]) {
    return storedLang;
  }

  // Fall back to device locale
  const deviceLocale = Localization.locale.split('-')[0];
  if (LANGUAGES[deviceLocale as LanguageCode]) {
    return deviceLocale;
  }

  // Default to English
  return 'en';
};

/**
 * Change application language
 */
export const changeLanguage = async (language: LanguageCode): Promise<void> => {
  await i18n.changeLanguage(language);
  await setStoredLanguage(language);
};

/**
 * Get current language code
 */
export const getCurrentLanguage = (): LanguageCode => {
  return i18n.language as LanguageCode;
};

/**
 * Get language name for display
 */
export const getLanguageName = (code: LanguageCode): string => {
  return LANGUAGES[code]?.name || code;
};

/**
 * Initialize i18n
 */
export const initializeI18n = async (): Promise<void> => {
  const initialLanguage = await detectLanguage();

  await i18n
    .use(initReactI18next)
    .init({
      resources,
      lng: initialLanguage,
      fallbackLng: 'en',
      interpolation: {
        escapeValue: false, // React already escapes values
      },
      react: {
        useSuspense: false, // Required for React Native
      },
      // Namespace configuration for better organization
      ns: ['translation'],
      defaultNS: 'translation',
      // Debug mode (disable in production)
      debug: __DEV__,
      // Missing key handling
      saveMissing: __DEV__,
      missingKeyHandler: (lng, ns, key) => {
        if (__DEV__) {
          console.warn(`Missing translation key: ${key} for language: ${lng}`);
        }
      },
    });
};

/**
 * Add a new language dynamically
 * Useful for loading languages from remote source
 */
export const addLanguage = (code: string, translations: Record<string, any>): void => {
  i18n.addResourceBundle(code, 'translation', translations, true, true);
};

/**
 * Check if language is supported
 */
export const isLanguageSupported = (code: string): boolean => {
  return code in LANGUAGES;
};

/**
 * Get list of available languages
 */
export const getAvailableLanguages = (): Array<{ code: LanguageCode; name: string; flag: string }> => {
  return Object.values(LANGUAGES);
};

// Export configured i18n instance
export default i18n;

// Re-export hooks for convenience
export { useTranslation } from 'react-i18next';
