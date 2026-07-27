"""Smoke tests for the dicelet Python module."""

import dicelet


def test_roll_basic():
    result = dicelet.roll("4d6k3")
    assert result.consumed == "4d6k3"
    assert len(result.values) == 1
    assert result.is_set is False


def test_roll_no_detail():
    result = dicelet.roll("4d6", show_detail=False)
    assert result.consumed == "4d6"
    # Without detail, full equals summary
    assert result.full == result.summary


def test_roll_set():
    result = dicelet.roll("6#4d6k3")
    assert result.is_set is True
    assert len(result.values) == 6


def test_roll_with_seed():
    r1 = dicelet.roll("10d100", seed=42)
    r2 = dicelet.roll("10d100", seed=42)
    assert r1.values == r2.values
    assert r1.full == r2.full


def test_roll_fault_tolerant():
    result = dicelet.roll("d20 + (d4+ test")
    assert "d20" in result.consumed
    assert len(result.tail) > 0


def test_parse():
    result = dicelet.parse("d20 + (d4+ test")
    assert result.success is True
    assert "d20" in result.consumed
    assert len(result.tail) > 0


def test_parse_invalid():
    result = dicelet.parse("hello")
    assert result.success is False


def test_rollresult_repr():
    result = dicelet.roll("1d6")
    assert repr(result).startswith("RollResult(")
    assert str(result) == result.full


def test_parseoutput_repr():
    result = dicelet.parse("1d6")
    assert repr(result).startswith("ParseOutput(")
