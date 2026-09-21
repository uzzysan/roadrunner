import React from 'react';
import { NavigationContainer } from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs';
import { Ionicons } from '@expo/vector-icons';
import { useTranslation } from 'react-i18next';

// Screens
import { HomeScreen } from '../screens/HomeScreen';
import { MapScreen } from '../screens/MapScreen';
import { TicketsScreen } from '../screens/TicketsScreen';
import { ProfileScreen } from '../screens/ProfileScreen';
import { LoginScreen } from '../screens/auth/LoginScreen';
import { RegisterScreen } from '../screens/auth/RegisterScreen';
import { StopDetailsScreen } from '../screens/StopDetailsScreen';
import RouteDetailsScreen from '../screens/RouteDetailsScreen';
import { BuyTicketScreen } from '../screens/BuyTicketScreen';
import { QRScannerScreen } from '../screens/QRScannerScreen';

// Store
import { useAuthStore } from '../store/authStore';
import { useTheme } from '../hooks/useTheme';
import type { MainTabParamList, RootStackParamList } from '../types';

export type { MainTabParamList, RootStackParamList } from '../types';

const Stack = createNativeStackNavigator<RootStackParamList>();
const Tab = createBottomTabNavigator<MainTabParamList>();

function MainTabs() {
  const { t } = useTranslation();
  const { colors, theme } = useTheme();
  const tabLabelTypography = theme.typography.label;

  return (
    <Tab.Navigator
      screenOptions={{
        headerShown: true,
        headerStyle: { backgroundColor: colors.surface },
        headerTintColor: colors.text,
        tabBarActiveTintColor: colors.primary,
        tabBarInactiveTintColor: colors.textSecondary,
        tabBarStyle: {
          backgroundColor: colors.surface,
          borderTopColor: colors.border,
          minHeight: theme.size.touchSafety,
        },
        tabBarLabelStyle: {
          fontSize: tabLabelTypography.size,
          fontWeight: String(tabLabelTypography.weight) as '600',
          lineHeight: tabLabelTypography.lineHeight,
        },
      }}
    >
      <Tab.Screen 
        name="Home" 
        component={HomeScreen}
        options={{
          title: t('navigation.home'),
          tabBarIcon: ({ color, focused, size }) => (
            <Ionicons color={color} name={focused ? 'home' : 'home-outline'} size={size} />
          ),
        }}
      />
      <Tab.Screen 
        name="Map" 
        component={MapScreen}
        options={{
          title: t('navigation.map'),
          tabBarIcon: ({ color, focused, size }) => (
            <Ionicons color={color} name={focused ? 'map' : 'map-outline'} size={size} />
          ),
        }}
      />
      <Tab.Screen 
        name="Tickets" 
        component={TicketsScreen}
        options={{
          title: t('navigation.tickets'),
          tabBarIcon: ({ color, focused, size }) => (
            <Ionicons color={color} name={focused ? 'ticket' : 'ticket-outline'} size={size} />
          ),
        }}
      />
      <Tab.Screen 
        name="Profile" 
        component={ProfileScreen}
        options={{
          title: t('navigation.profile'),
          tabBarIcon: ({ color, focused, size }) => (
            <Ionicons color={color} name={focused ? 'person' : 'person-outline'} size={size} />
          ),
        }}
      />
    </Tab.Navigator>
  );
}

export function AppNavigator() {
  const { isAuthenticated } = useAuthStore();
  const { theme } = useTheme();

  return (
    <NavigationContainer theme={theme.navigationTheme}>
      <Stack.Navigator screenOptions={{ headerShown: false }}>
        {isAuthenticated ? (
          <>
            <Stack.Screen name="Main" component={MainTabs} />
            <Stack.Screen name="StopDetails" component={StopDetailsScreen} />
            <Stack.Screen name="RouteDetails" component={RouteDetailsScreen} />
            <Stack.Screen name="BuyTicket" component={BuyTicketScreen} />
            <Stack.Screen name="QRScanner" component={QRScannerScreen} />
          </>
        ) : (
          <>
            <Stack.Screen name="Login" component={LoginScreen} />
            <Stack.Screen name="Register" component={RegisterScreen} />
          </>
        )}
      </Stack.Navigator>
    </NavigationContainer>
  );
}
