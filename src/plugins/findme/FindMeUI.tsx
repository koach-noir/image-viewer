import React, { useState, useEffect } from 'react';
import FindMePlugin from './FindMePlugin';
import getEventSystem from '../../core/EventSystem';

// スタイル定義
const styles = {
  container: {
    display: 'flex',
    flexDirection: 'column' as const,
    alignItems: 'center',
    justifyContent: 'center',
    height: '100%',
    backgroundColor: '#f4f4f4',
    fontFamily: 'Arial, sans-serif',
  },
  title: {
    color: '#4a90e2',
    fontSize: '2.5rem',
    marginBottom: '20px',
  },
  subtitle: {
    color: '#666',
    fontSize: '1.2rem',
    marginBottom: '30px',
  },
  button: {
    padding: '10px 20px',
    fontSize: '1rem',
    backgroundColor: '#4a90e2',
    color: 'white',
    border: 'none',
    borderRadius: '5px',
    cursor: 'pointer',
    marginBottom: '10px',
  },
  difficultySelect: {
    padding: '8px',
    fontSize: '1rem',
    marginBottom: '20px',
  }
};

const FindMeUI: React.FC = () => {
  const [gameState, setGameState] = useState<string>('ready');
  const [difficulty, setDifficulty] = useState<'easy' | 'medium' | 'hard'>('easy');

  // プラグインイベントの購読
  useEffect(() => {
    const eventSystem = getEventSystem();

    // ゲーム開始イベントの購読
    const gameStartHandler = (event: any) => {
      setGameState('in_progress');
      console.log('Game started:', event);
    };

    // 難易度変更イベントの購読
    const difficultyChangeHandler = (event: any) => {
      setDifficulty(event.difficulty);
      console.log('Difficulty changed:', event);
    };

    // イベントの追加
    eventSystem.subscribe('findme:game_started', gameStartHandler);
    eventSystem.subscribe('findme:difficulty_changed', difficultyChangeHandler);

    // クリーンアップ関数
    return () => {
      eventSystem.unsubscribe('findme:game_started', gameStartHandler);
      eventSystem.unsubscribe('findme:difficulty_changed', difficultyChangeHandler);
    };
  }, []);

  // ゲーム開始ハンドラ
  const handleStartGame = async () => {
    try {
      const startGameHandler = FindMePlugin.getApiHandlers()
        .find(handler => handler.name === 'start_game');
      if (startGameHandler) {
        await startGameHandler.handler({ difficulty: 'easy' });
      }
    } catch (error) {
      console.error('Failed to start game:', error);
    }
  };

  // 難易度変更ハンドラ
  const handleDifficultyChange = async (newDifficulty: 'easy' | 'medium' | 'hard') => {
    try {
      const setDifficultyHandler = FindMePlugin.getApiHandlers()
        .find(handler => handler.name === 'set_difficulty');
      
      if (setDifficultyHandler) {
        await setDifficultyHandler.handler({ difficulty: newDifficulty });
      }
    } catch (error) {
      console.error('Failed to set difficulty:', error);
    }
  };

  return (
    <div style={styles.container}>
      <h1 style={styles.title}>Hello FindMe</h1>
      <p style={styles.subtitle}>Welcome to the FindMe Image Game!</p>
      
      {/* 難易度選択 */}
      <select 
        style={styles.difficultySelect}
        value={difficulty}
        onChange={(e) => handleDifficultyChange(e.target.value as 'easy' | 'medium' | 'hard')}
      >
        <option value="easy">Easy</option>
        <option value="medium">Medium</option>
        <option value="hard">Hard</option>
      </select>

      {/* ゲーム開始ボタン */}
      <button 
        style={styles.button}
        onClick={handleStartGame}
        disabled={gameState === 'in_progress'}
      >
        {gameState === 'ready' ? 'Start Game' : 'Game in Progress'}
      </button>
    </div>
  );
};

export default FindMeUI;
