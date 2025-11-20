import React, { ReactNode } from 'react';
import { Loader2 } from 'lucide-react';
import { StandardTooltip } from './tooltip';

interface ToolbarButtonProps {
  onClick: () => void;
  disabled?: boolean;
  isLoading?: boolean;
  loadingText?: string;
  children: ReactNode;
  tooltip: string;
  variant?: 'primary' | 'secondary' | 'danger';
  className?: string;
  isAnyLoading?: boolean;
}

const ToolbarButton: React.FC<ToolbarButtonProps> = ({
  onClick,
  disabled = false,
  isLoading = false,
  loadingText,
  children,
  tooltip,
  variant = 'secondary',
  className = '',
  isAnyLoading = false
}) => {
  const buttonClasses = `
    btn btn-${variant} px-2 py-1 text-sm font-medium whitespace-nowrap
    transition-all duration-200 transform hover:scale-105 active:scale-95
    ${isLoading ? 'cursor-wait opacity-80' : ''}
    ${disabled || isLoading || isAnyLoading ? 'cursor-not-allowed opacity-60' : 'hover:shadow-lg'}
    ${className}
  `.trim();

  const LoadingSpinner = () => (
    <Loader2 className="animate-spin h-4 w-4 mr-2" />
  );

  const buttonElement = (
    <button
      className={buttonClasses}
      onClick={onClick}
      disabled={disabled || isLoading || isAnyLoading}
    >
      {isLoading ? (
        <>
          <LoadingSpinner />
          <span>{loadingText || '处理中...'}</span>
        </>
      ) : (
        children
      )}
    </button>
  );

  return (
    <StandardTooltip
      content={tooltip}
      side="top"
      delayDuration={300}
      className="max-w-xs z-50"
    >
      {buttonElement}
    </StandardTooltip>
  );
};

export default ToolbarButton;