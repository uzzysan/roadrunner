import React from 'react';
import { useTranslation } from 'react-i18next';

import { EmptyState, Screen } from '../components';

export default function TrackingScreen() {
  const { t } = useTranslation();
  return <Screen><EmptyState description={t('transit_tracking.unavailable_description')} icon="bus-outline" title={t('transit_tracking.unavailable_title')} /></Screen>;
}
