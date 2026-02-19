import React from 'react';
import { StatusBar } from 'expo-status-bar';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import { PaperProvider } from 'react-native-paper';
import { NavigationContainer } from '@react-navigation/native';
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs';
import { MaterialCommunityIcons } from '@expo/vector-icons';

import { lightTheme, darkTheme } from './src/theme/colors';
import HomeScreen from './src/screens/HomeScreen';
import TicketsScreen from './src/screens/TicketsScreen';
import TrackingScreen from './src/screens/TrackingScreen';
import ProfileScreen from './src/screens/ProfileScreen';

const Tab = createBottomTabNavigator();

export default function App() {
  // TODO: Get from system or user preference
  const isDarkMode = false;
  const theme = isDarkMode ? darkTheme : lightTheme;

  return (
    <SafeAreaProvider>
      <PaperProvider theme={theme}>
        <NavigationContainer>
          <Tab.Navigator
            screenOptions={({ route }) => ({
              tabBarIcon: ({ focused, color, size }) => {
                let iconName: string;

                if (route.name === 'Home') {
                  iconName = focused ? 'home' : 'home-outline';
                } else if (route.name === 'Tickets') {
                  iconName = focused ? 'ticket' : 'ticket-outline';
                } else if (route.name === 'Tracking') {
                  iconName = focused ? 'map-marker' : 'map-marker-outline';
                } else if (route.name === 'Profile') {
                  iconName = focused ? 'account' : 'account-outline';
                } else {
                  iconName = 'help';
                }

                return <MaterialCommunityIcons name={iconName as any} size={size} color={color} />;
              },
              tabBarActiveTintColor: theme.colors.primary,
              tabBarInactiveTintColor: theme.colors.onSurfaceVariant,
              tabBarStyle: {
                backgroundColor: theme.colors.surface,
                borderTopColor: theme.colors.outlineVariant,
              },
              headerStyle: {
                backgroundColor: theme.colors.primary,
              },
              headerTintColor: theme.colors.onPrimary,
            })}
          >
            <Tab.Screen 
              name="Home" 
              component={HomeScreen}
              options={{ title: 'RoadRunner' }}
            />
            <Tab.Screen 
              name="Tickets" 
              component={TicketsScreen}
              options={{ title: 'Bilety' }}
            />
            <Tab.Screen 
              name="Tracking" 
              component={TrackingScreen}
              options={{ title: 'Śledzenie' }}
            />
            <Tab.Screen 
              name="Profile" 
              component={ProfileScreen}
              options={{ title: 'Profil' }}
            />
          </Tab.Navigator>
        </NavigationContainer>
        <StatusBar style={isDarkMode ? 'light' : 'dark'} />
      </PaperProvider>
    </SafeAreaProvider>
  );
}
