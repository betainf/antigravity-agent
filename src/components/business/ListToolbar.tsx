import React from 'react';
import { ArrowUpDown, Search, X } from 'lucide-react';
import { cn } from '@/lib/utils.ts';
import { BaseInput } from '@/components/base-ui/BaseInput';
import type { UserTier } from '@/modules/use-account-addition-data.ts';
import { Select as AntSelect } from 'antd';

export type ListSortKey = 'name' | 'claude' | 'gemini' | 'tier';
export type ListToolbarValue = {
  query: string;
  sortKey: ListSortKey;
};

const defaultSortOptions: Array<{ value: ListSortKey; label: string }> = [
  { value: 'name', label: '用户名首字母' },
  { value: 'claude', label: 'Claude 配额' },
  { value: 'gemini', label: 'Gemini 配额' },
  { value: 'tier', label: '层次' },
];

const tierUiMap: Record<UserTier, { label: string; accentClass: string }> = {
  'free-tier': {
    label: 'Free',
    accentClass: 'text-slate-900 dark:text-slate-50',
  },
  'g1-pro-tier': {
    label: 'Pro',
    accentClass: 'text-amber-700 dark:text-amber-300',
  },
  'g1-ultra-tier': {
    label: 'Ultra',
    accentClass: 'text-violet-700 dark:text-violet-300',
  },
};

const allTiers: UserTier[] = ['free-tier', 'g1-pro-tier', 'g1-ultra-tier'];

export interface BusinessListToolbarProps {
  /** 列表总数 */
  total: number;
  /** 搜索关键字 */
  query: string;
  /** 排序 key */
  sortKey: ListSortKey;
  /** 任一项变更时回调（返回完整状态） */
  onChange: (next: ListToolbarValue) => void;
  className?: string;
}

/**
 * Business Component: ListToolbar
 * 列表顶部工具栏（标题 + 搜索 + 自定义动作/过滤器插槽）
 */
const BusinessListToolbar: React.FC<BusinessListToolbarProps> = ({
  total,
  query,
  sortKey,
  onChange,
  className,
}) => {
  const titleText = '账户备份';
  const [internalTiers, setInternalTiers] = React.useState<UserTier[]>([]);

  const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange({ query: e.target.value, sortKey });
  };

  const handleClearSearch = () => {
    onChange({ query: '', sortKey });
  };

  const handleSortChange = (next: ListSortKey) => {
    onChange({ query, sortKey: next });
  };

  const toggleTier = (tier: UserTier) => {
    const exists = internalTiers.includes(tier);
    const nextTiers = exists
      ? internalTiers.filter(t => t !== tier)
      : [...internalTiers, tier];
    setInternalTiers(nextTiers);
  };

  const clearTiers = () => {
    setInternalTiers([]);
  };

  const containerClasses = [
    'flex items-center justify-between gap-3 px-3 py-2 rounded-xl border',
    'bg-white/80 dark:bg-slate-900/60 border-slate-200 dark:border-slate-700',
    'backdrop-blur-sm shadow-sm',
  ];

  const titleClass = 'text-sm font-semibold text-slate-900 dark:text-slate-50';
  const metaClass = 'text-xs text-slate-500 dark:text-slate-400';

  return (
    <div className={cn(...containerClasses, className)}>
      <div className="flex items-center gap-3 min-w-0">
        <div className="flex items-center gap-2 min-w-0">
          <h2 className={cn(titleClass, 'truncate')}>{titleText}</h2>
          <span className={cn(metaClass, 'whitespace-nowrap')}>共 {total} 条</span>
        </div>

        <BaseInput
          value={query}
          onChange={handleSearchChange}
          placeholder="搜索邮箱或昵称..."
          leftIcon={<Search className="h-4 w-4" />}
          rightIcon={
            query ? (
              <button
                type="button"
                onClick={handleClearSearch}
                className="hover:text-slate-600 dark:hover:text-slate-300 transition-colors"
                aria-label="Clear search"
              >
                <X className="h-4 w-4" />
              </button>
            ) : undefined
          }
          containerClassName="w-64 !space-y-0 ml-2"
          className="py-1.5 h-8 text-sm"
        />
      </div>

      <div className="flex items-center gap-2 shrink-0">
        {/* 层次筛选：分段按钮 */}
        <div
          className={cn(
            'flex items-center gap-0.5 p-0.5 rounded-lg border',
            'bg-slate-50 dark:bg-slate-800/60 border-slate-200 dark:border-slate-700'
          )}
        >
          <button
            type="button"
            onClick={clearTiers}
            className={cn(
              'px-2.5 py-1 rounded-md text-xs font-medium transition-colors',
              internalTiers.length === 0
                ? 'bg-white dark:bg-slate-900 text-blue-700 dark:text-blue-300 shadow-sm'
                : 'text-slate-600 dark:text-slate-300 hover:bg-white/70 dark:hover:bg-slate-900/60'
            )}
          >
            全部
          </button>
          {allTiers.map(tier => {
            const isActive = internalTiers.includes(tier);
            const { label, accentClass } = tierUiMap[tier];
            return (
              <button
                key={tier}
                type="button"
                onClick={() => toggleTier(tier)}
                className={cn(
                  'px-2.5 py-1 rounded-md text-xs font-medium transition-colors',
                  isActive
                    ? cn('bg-white dark:bg-slate-900 shadow-sm', accentClass)
                    : 'text-slate-600 dark:text-slate-300 hover:bg-white/70 dark:hover:bg-slate-900/60'
                )}
              >
                {label}
              </button>
            );
          })}
        </div>

        {/* 排序选择：紧凑胶囊 */}
        <div
          className={cn(
            'flex items-center gap-1 h-8 px-2 rounded-lg border',
            'bg-slate-50 dark:bg-slate-800/60 border-slate-200 dark:border-slate-700'
          )}
        >
          <ArrowUpDown className="h-3.5 w-3.5 text-slate-400" />
          <AntSelect
            value={sortKey}
            onChange={(v) => handleSortChange(v as ListSortKey)}
            size="small"
            variant="borderless"
            popupMatchSelectWidth={false}
            options={defaultSortOptions.map(opt => ({
              value: opt.value,
              label: opt.label,
            }))}
            className={cn(
              'min-w-[120px]',
              '[&_.ant-select-selection-item]:text-xs'
            )}
          />
        </div>
      </div>
    </div>
  );
};

export default BusinessListToolbar;
