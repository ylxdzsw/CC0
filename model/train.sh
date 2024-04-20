set -x

while :
do
    python train2.py model1 model2
    python train2.py model2 model3
    python train2.py model3 model4
done
