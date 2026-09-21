import React from 'react';
import { useTranslation } from 'react-i18next';
import type { ViewProps } from 'react-native';
import { EmptyState } from './EmptyState';

export interface ErrorStateProps extends ViewProps {
  title?: string;
  description: string;
  retryLabel?: string;
  onRetry?: () => void;
}

export function ErrorState({ title, description, retryLabel, onRetry, ...props }: ErrorStateProps) {
  const { t } = useTranslation();
  return (
    <EmptyState
      accessibilityLiveRegion="polite"
      actionLabel={onRetry ? retryLabel ?? t('common.retry') : undefined}
      description={description}
      icon="alert-circle-outline"
      onAction={onRetry}
      title={title ?? t('common.error')}
      {...props}
    />
  );
}
