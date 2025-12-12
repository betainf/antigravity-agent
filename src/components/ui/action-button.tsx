import React, { ReactNode } from 'react';
import { Loader2 } from 'lucide-react';
import { StandardTooltip } from './tooltip';
import { cn } from '@/lib/utils.ts';

export interface ActionButtonProps {
    // 基础属性
    onClick: () => void;
    children: ReactNode;

    // 状态
    isLoading?: boolean;
    disabled?: boolean;

    // 样式
    variant?: 'primary' | 'secondary' | 'danger' | 'outline' | 'ghost';
    size?: 'sm' | 'default' | 'lg';

    // 图标和提示
    icon?: ReactNode;
    tooltip?: string;
    loadingText?: string;

    // 其他
    className?: string;
    isAnyLoading?: boolean;
}

/**
 * 统一的操作按钮组件
 * 整合了 loading、tooltip、icon 等功能
 */
const ActionButton: React.FC<ActionButtonProps> = ({
    onClick,
    children,
    isLoading = false,
    disabled = false,
    variant = 'secondary',
    size = 'default',
    icon,
    tooltip,
    loadingText,
    className = '',
    isAnyLoading = false
}) => {
    // 变体样式
    const variantClasses = {
        primary: 'bg-blue-600 hover:bg-blue-700 text-white shadow-md hover:shadow-xl',
        secondary: 'bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-900 dark:text-gray-100',
        danger: 'bg-red-600 hover:bg-red-700 text-white',
        outline: 'border border-gray-300 dark:border-gray-600 bg-transparent hover:bg-gray-100 dark:hover:bg-gray-800',
        ghost: 'bg-transparent hover:bg-gray-100 dark:hover:bg-gray-800'
    };

    // 尺寸样式
    const sizeClasses = {
        sm: 'px-2 py-1 text-xs',
        default: 'px-3 py-2 text-sm',
        lg: 'px-4 py-3 text-base'
    };

    const buttonClasses = cn(
        // 基础样式
        'inline-flex items-center justify-center gap-2',
        'rounded-lg font-medium whitespace-nowrap',
        'transition-all duration-200',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2',

        // 变体和尺寸
        variantClasses[variant],
        sizeClasses[size],

        // 状态样式
        isLoading && 'cursor-wait opacity-80',
        (disabled || isLoading || isAnyLoading) && 'cursor-not-allowed opacity-60',
        !(disabled || isLoading || isAnyLoading) && 'hover:scale-105 active:scale-95',

        // 自定义类名
        className
    );

    const buttonContent = isLoading ? (
        <>
            <Loader2 className="h-4 w-4 animate-spin" />
            <span>{loadingText || '处理中...'}</span>
        </>
    ) : (
        <>
            {icon && <span className="flex items-center">{icon}</span>}
            <span>{children}</span>
        </>
    );

    const button = (
        <button
            className={buttonClasses}
            onClick={onClick}
            disabled={disabled || isLoading || isAnyLoading}
            type="button"
        >
            {buttonContent}
        </button>
    );

    // 如果有 tooltip，包装在 Tooltip 中
    if (tooltip) {
        return (
            <StandardTooltip
                content={tooltip}
                side="top"
                delayDuration={300}
                className="max-w-xs z-50"
            >
                {button}
            </StandardTooltip>
        );
    }

    return button;
};

export default ActionButton;
