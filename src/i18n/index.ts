import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

// Import translation files
import commonEN from './locales/en/common.json';
import commonZhCN from './locales/zh-CN/common.json';
import commonZhTW from './locales/zh-TW/common.json';

import dashboardEN from './locales/en/dashboard.json';
import dashboardZhCN from './locales/zh-CN/dashboard.json';
import dashboardZhTW from './locales/zh-TW/dashboard.json';

import accountEN from './locales/en/account.json';
import accountZhCN from './locales/zh-CN/account.json';
import accountZhTW from './locales/zh-TW/account.json';

import settingsEN from './locales/en/settings.json';
import settingsZhCN from './locales/zh-CN/settings.json';
import settingsZhTW from './locales/zh-TW/settings.json';

import notificationsEN from './locales/en/notifications.json';
import notificationsZhCN from './locales/zh-CN/notifications.json';
import notificationsZhTW from './locales/zh-TW/notifications.json';

import importExportEN from './locales/en/import-export.json';
import importExportZhCN from './locales/zh-CN/import-export.json';
import importExportZhTW from './locales/zh-TW/import-export.json';

const resources = {
  en: {
    common: commonEN,
    dashboard: dashboardEN,
    account: accountEN,
    settings: settingsEN,
    notifications: notificationsEN,
    'import-export': importExportEN,
  },
  'zh-CN': {
    common: commonZhCN,
    dashboard: dashboardZhCN,
    account: accountZhCN,
    settings: settingsZhCN,
    notifications: notificationsZhCN,
    'import-export': importExportZhCN,
  },
  'zh-TW': {
    common: commonZhTW,
    dashboard: dashboardZhTW,
    account: accountZhTW,
    settings: settingsZhTW,
    notifications: notificationsZhTW,
    'import-export': importExportZhTW,
  },
};

// Initialize with default language, will be updated from Tauri later
let cachedLanguage: string | null = null;

// Custom language detector that uses cached value from Tauri
const TauriLanguageDetector = {
  name: 'tauriLanguageDetector',
  lookup: (): string | undefined => {
    return cachedLanguage || undefined;
  },
  cacheUserLanguage: async (lng: string): Promise<void> => {
    cachedLanguage = lng;
    try {
      const { SettingsCommands } = await import('@/commands/SettingsCommands');
      await SettingsCommands.setLanguage(lng);
    } catch (error) {
      console.error('Failed to save language to Tauri:', error);
    }
  },
};

// Load language from Tauri asynchronously after initialization
async function loadLanguageFromTauri() {
  try {
    const { SettingsCommands } = await import('@/commands/SettingsCommands');
    const settings = await SettingsCommands.getAll();
    if (settings?.language) {
      cachedLanguage = settings.language;
      await i18n.changeLanguage(settings.language);
    }
  } catch (error) {
    console.error('Failed to load language from Tauri:', error);
  }
}

i18n
  // Detect user language
  .use(LanguageDetector)
  // Pass the i18n instance to react-i18next
  .use(initReactI18next)
  // Initialize i18next
  .init({
    resources,
    fallbackLng: 'zh-CN', // Default to Chinese (current behavior)
    defaultNS: 'common',
    ns: ['common', 'dashboard', 'account', 'settings', 'notifications', 'import-export'],
    
    interpolation: {
      escapeValue: false, // React already escapes
    },
    
    detection: {
      // Detection order (tauriLanguageDetector won't block since it's synchronous now)
      order: [
        'navigator',    // Browser language
        'htmlTag',      // HTML tag
      ],
      caches: [], // Don't use localStorage (we use Tauri)
    },
    
    react: {
      useSuspense: false, // Disable suspense
      bindI18n: 'languageChanged',
      bindI18nStore: '',
      transEmptyNodeValue: '',
      transSupportBasicHtmlNodes: true,
      transKeepBasicHtmlNodesFor: ['br', 'strong', 'i', 'p'],
    },
    
    // Development features
    debug: import.meta.env.DEV,
    saveMissing: import.meta.env.DEV,
    missingKeyHandler: (lng, ns, key) => {
      if (import.meta.env.DEV) {
        console.warn(`Missing translation: [${lng}] ${ns}:${key}`);
      }
    },
  });

// Load language from Tauri after i18n is initialized
loadLanguageFromTauri();

export default i18n;
