using UnityEngine;

public class CharacterMovement : MonoBehaviour
{
    private Vector3 _movementInput;
    private Vector3 _mousePosition;

    [SerializeField] float epsilon = 0.1f;

    private Rigidbody _rb;
    private Animator _animator;

    public float Speed = 5f;
    [Range(0f, 1f)] public float SmoothRotation = 0.5f;

    Vector3 GetMousePosition()
    {
        Ray ray = Camera.main.ScreenPointToRay(Input.mousePosition);
        Plane plane = new Plane(Vector3.up, transform.position);
        if (plane.Raycast(ray, out float distance)) return ray.GetPoint(distance);
        else return transform.position;
    }

    private void Reset()
    {
        _rb = transform.GetComponent<Rigidbody>();
        _animator = _rb.GetComponent<Animator>();
    }

    void Update()
    {
        _mousePosition = GetMousePosition();
        _movementInput = new Vector3(Input.GetAxis("Horizontal"), 0, Input.GetAxis("Vertical"));

        // animator
        if (_movementInput.z > epsilon) _animator.SetBool("IsIdle", false);
        else _animator.SetBool("IsIdle", true);

        // go to FixedUpdates
        _rb.MovePosition(_rb.position + _rb.rotation * _movementInput * Speed * Time.deltaTime);
        _rb.MoveRotation(Quaternion.Slerp(_rb.rotation, _rb.rotation * Quaternion.FromToRotation(transform.forward, _mousePosition - transform.position).normalized, Time.deltaTime * 50 * SmoothRotation));
        transform.rotation = Quaternion.Euler(0, transform.rotation.eulerAngles.y, 0);
    }

    private void FixedUpdate()
    {

    }
}
