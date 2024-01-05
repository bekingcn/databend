
# gen tpch data to ./benchmark/tpch/data
./scripts/setup/dev_setup.sh -t 0.1

# start minikube
minikube start --cpus='4' --memory='14G'

# setup databend with distribution configs under ./scripts/kubernetes
kubectl create ns tenant1
kubectl apply -f scripts/kubernetes/minio-sample.yaml
kubectl apply -f scripts/kubernetes/meta-standalone.yaml
# NOTE: ENV should be META_ENDPOINTS, not META_SERVICE
# a bunch of changes from original query-cluster.yaml
kubectl apply -f scripts/kubernetes/query-cluster-working.yaml


# download bendsql by **VERSION**
curl -L -o ./bendsql-x86_64-unknown-linux-gnu.tar.gz https://github.com/datafuselabs/bendsql/releases/download/v0.12.1/bendsql-x86_64-unknown-linux-gnu.tar.gz
tar -xvzf bendsql-x86_64-unknown-linux-gnu.tar.gz

# port-forward
nohup kubectl port-forward -n tenant1 svc/query-service 8000:8000 > pf8000.out &
# ./bendsql -uroot --host localhost --port 8000 --database tpch --quote-style=never
./bendsql 

# sql 
# SELECT avg(number) FROM numbers(100000000);


# minio
nohup kubectl port-forward svc/minio 9000:9000 > pf9000.out &
export MC_HOST_minio_local=http://minio:minio123@localhost:9000
mc mb minio_local/tpch
mc mb minio_local/sample-storage
# upload 
mc cp benchmark/tpch/data/*.tbl minio_local/sample-storage

# create tpch db and tables, them copy data into tables...
chmod +x ./prepare_table-tpch.sh
./prepare_table-tpch.sh


## SQL
## Q1
```
select 
    l_returnflag,
    l_linestatus, 
    sum(l_quantity) as sum_qty,
    sum(l_extendedprice) as sum_base_price,
    sum(l_extendedprice * (1 - l_discount)) as sum_disc_price, 
    sum(l_extendedprice * (1 - l_discount) * (1 + l_tax)) as sum_charge, 
    avg(l_quantity) as avg_qty, 
    avg(l_extendedprice) as avg_price, 
    avg(l_discount) as avg_disc, 
    count(*) as count_order
from 
    lineitem
where 
    l_shipdate <= date'1998-09-01'
group by 
    l_returnflag, 
    l_linestatus
order by 
    l_returnflag, 
    l_linestatus;
```