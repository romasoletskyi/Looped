using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public delegate void GameEvent();

public enum GameState
{
    Running,
    Starting,
    Ending
}

public class Clock : MonoBehaviour
{
    public float RunTime { get; set; } = 30;
    public GameState State { get; private set; } = GameState.Starting;

    private PriorityQueue<GameEvent, float> endEvents;
    private PriorityQueue<GameEvent, float> startEvents;

    private float _startTime;
    private float _deltaRunTime;

    // Start is called before the first frame update
    private void Reset()
    {
        endEvents = new PriorityQueue<GameEvent, float>();
        startEvents = new PriorityQueue<GameEvent, float>();
    }

    // Update is called once per frame
    private void Update()
    {
        if (State == GameState.Running && Time.fixedTime - _startTime > _deltaRunTime)
        {
            foreach (GameEvent e in endEvents) e();
            StartCoroutine(Ending());
        }
        else if (State == GameState.Starting)
        {
            foreach (GameEvent e in startEvents) e();
            _startTime = Time.fixedTime;
            _deltaRunTime = RunTime;
            State = GameState.Running;
        } 
    }

    private IEnumerator Ending()
    {
        yield return new WaitForSeconds(1f);
        State = GameState.Starting;
        yield return null;
    }

    public void addStartEvent (GameEvent e) {startEvents.Add(e);} 
    public void addEndEvent(GameEvent e) { endEvents.Add(e); } 

}
