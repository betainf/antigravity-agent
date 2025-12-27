import React from 'react';
import { useTranslation } from 'react-i18next';
import { Select } from 'antd';
import { languages, type SupportedLanguage } from '@/i18n/types';
import { SettingsCommands } from '@/commands/SettingsCommands';
import { logger } from '@/lib/logger';
import toast from 'react-hot-toast';
import dayjs from 'dayjs';

interface LanguageSwitcherProps {
  className?: string;
  showNativeName?: boolean;
}

export const LanguageSwitcher: React.FC<LanguageSwitcherProps> = ({
  className,
  showNativeName = true,
}) => {
  const { i18n } = useTranslation();
  const [loading, setLoading] = React.useState(false);

  const currentLanguage = i18n.language as SupportedLanguage;

  const handleLanguageChange = async (newLanguage: SupportedLanguage) => {
    if (newLanguage === currentLanguage) return;

    setLoading(true);
    try {
      // Change language in i18next
      await i18n.changeLanguage(newLanguage);
      
      // Persist to Tauri backend
      await SettingsCommands.setLanguage(newLanguage);
      
      // Update dayjs locale
      const localeMap: Record<SupportedLanguage, string> = {
        'en': 'en',
        'zh-CN': 'zh-cn',
        'zh-TW': 'zh-tw',
      };
      dayjs.locale(localeMap[newLanguage]);
      
      toast.success('Language changed successfully');
      
      logger.info('Language changed', {
        module: 'LanguageSwitcher',
        from: currentLanguage,
        to: newLanguage,
      });
    } catch (error) {
      toast.error('Failed to change language');
      logger.error('Failed to change language', {
        module: 'LanguageSwitcher',
        error: error instanceof Error ? error.message : String(error),
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <Select
      className={className}
      value={currentLanguage}
      onChange={handleLanguageChange}
      loading={loading}
      style={{ width: 200 }}
      options={languages.map((lang) => ({
        value: lang.code,
        label: (
          <span>
            <span>{lang.flag}</span>
            <span className="ml-2">
              {showNativeName ? lang.nativeName : lang.name}
            </span>
          </span>
        ),
      }))}
    />
  );
};
